use crate::types::pipewire::{AudioDevice, DeviceType, LinkInfo, Port};
use anyhow::{Result, anyhow};
use pipewire::{
    context::ContextRc, link::Link, main_loop::MainLoopRc, properties::properties,
    registry::GlobalObject, spa::utils::dict::DictRef,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::OnceLock, thread};
use tokio::sync::oneshot;

/// Every audio node PWSP knows about, split by role.
pub struct AllDevices {
    /// `Audio/Source*` nodes — real and virtual microphones.
    pub inputs: Vec<AudioDevice>,
    /// `Stream/Output/Audio` nodes — application playback streams, including our own.
    pub outputs: Vec<AudioDevice>,
    /// `Audio/Sink` nodes — speakers and headphones.
    pub sinks: Vec<AudioDevice>,
}

impl AllDevices {
    pub fn iter(&self) -> impl Iterator<Item = &AudioDevice> {
        self.inputs
            .iter()
            .chain(self.outputs.iter())
            .chain(self.sinks.iter())
    }
}

pub enum PwCommand {
    GetDevices {
        resp: oneshot::Sender<AllDevices>,
    },
    GetLinks {
        resp: oneshot::Sender<Vec<LinkInfo>>,
    },
    CreateVirtualMic {
        resp: oneshot::Sender<Result<u32, String>>,
    },
    CreateLink {
        output_fl: Port,
        output_fr: Port,
        input_fl: Port,
        input_fr: Port,
        resp: oneshot::Sender<Result<(u32, u32), String>>,
    },
    /// Drops a proxy we created ourselves, which destroys the underlying object.
    DestroyObject {
        id: u32,
    },
    /// Destroys a global object owned by somebody else, such as an auto-created link.
    DestroyGlobal {
        id: u32,
    },
}

struct AppState {
    input_devices: HashMap<u32, AudioDevice>,
    output_devices: HashMap<u32, AudioDevice>,
    sink_devices: HashMap<u32, AudioDevice>,
    links: HashMap<u32, LinkInfo>,
    ports: HashMap<u32, Port>,
    proxies: HashMap<u32, Box<dyn std::any::Any>>,
    proxy_id_counter: u32,
    ready_tx: Option<std::sync::mpsc::Sender<()>>,
}

pub struct PipewireManager {
    pub sender: pipewire::channel::Sender<PwCommand>,
}

static MANAGER: OnceLock<PipewireManager> = OnceLock::new();

pub fn get_manager() -> &'static PipewireManager {
    MANAGER.get_or_init(|| {
        let (pw_sender, pw_receiver) = pipewire::channel::channel::<PwCommand>();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            let (main_loop, context) = setup_pipewire_context().expect("Failed to setup pipewire");

            // Leak main_loop and context so their borrows can be 'static
            let main_loop = Box::leak(Box::new(main_loop));
            let context = Box::leak(Box::new(context));

            // Leak to fix lifetime issues since this thread lives forever. Shared rather
            // than mutable so both `core` and the `registry` borrowed from it can be
            // captured by the command closure below.
            let core: &'static _ = Box::leak(Box::new(
                context
                    .connect(None)
                    .expect("Failed to connect to pipewire"),
            ));
            let registry: &'static _ = Box::leak(Box::new(
                core.get_registry().expect("Failed to get registry"),
            ));

            let state = Rc::new(RefCell::new(AppState {
                input_devices: HashMap::new(),
                output_devices: HashMap::new(),
                sink_devices: HashMap::new(),
                links: HashMap::new(),
                ports: HashMap::new(),
                proxies: HashMap::new(),
                proxy_id_counter: 10000,
                ready_tx: Some(ready_tx),
            }));

            let state_for_registry_add = state.clone();
            let state_for_registry_remove = state.clone();

            let _listener = registry
                .add_listener_local()
                .global(move |global| {
                    let mut s = state_for_registry_add.borrow_mut();
                    match parse_global_object(global) {
                        ParsedGlobal::Device(device) => match device.device_type {
                            DeviceType::Input => {
                                s.input_devices.insert(device.id, device);
                            }
                            DeviceType::Output => {
                                s.output_devices.insert(device.id, device);
                            }
                            DeviceType::Sink => {
                                s.sink_devices.insert(device.id, device);
                            }
                        },
                        ParsedGlobal::Port(port) => {
                            let node_id = port.node_id;
                            s.ports.insert(port.port_id, port.clone());
                            if let Some(d) = s.input_devices.get_mut(&node_id) {
                                d.add_port(port);
                            } else if let Some(d) = s.output_devices.get_mut(&node_id) {
                                d.add_port(port);
                            } else if let Some(d) = s.sink_devices.get_mut(&node_id) {
                                d.add_port(port);
                            }
                        }
                        ParsedGlobal::Link(link) => {
                            s.links.insert(link.id, link);
                        }
                        ParsedGlobal::Unknown => {}
                    }
                })
                .global_remove(move |id| {
                    let mut s = state_for_registry_remove.borrow_mut();
                    s.input_devices.remove(&id);
                    s.output_devices.remove(&id);
                    s.sink_devices.remove(&id);
                    s.links.remove(&id);
                    s.links
                        .retain(|_, link| link.output_node != id && link.input_node != id);
                    s.ports.retain(|_, port| port.node_id != id);
                    s.ports.remove(&id);
                })
                .register();

            // sync to signal ready
            let state_for_sync = state.clone();
            let _core_listener = core
                .add_listener_local()
                .done(move |id, _seq| {
                    if id == 0 {
                        let mut s = state_for_sync.borrow_mut();
                        if let Some(tx) = s.ready_tx.take() {
                            let _ = tx.send(());
                        }
                    }
                })
                .register();

            let _pending = core.sync(0).expect("sync failed");

            let state_for_cmd = state.clone();
            let _receiver = pw_receiver.attach(main_loop.loop_(), move |cmd| {
                let mut s = state_for_cmd.borrow_mut();
                match cmd {
                    PwCommand::GetDevices { resp } => {
                        let mut inputs: Vec<AudioDevice> =
                            s.input_devices.values().cloned().collect();
                        let mut outputs: Vec<AudioDevice> =
                            s.output_devices.values().cloned().collect();
                        let mut sinks: Vec<AudioDevice> =
                            s.sink_devices.values().cloned().collect();
                        inputs.sort_by_key(|a| a.id);
                        outputs.sort_by_key(|a| a.id);
                        sinks.sort_by_key(|a| a.id);
                        let _ = resp.send(AllDevices {
                            inputs,
                            outputs,
                            sinks,
                        });
                    }
                    PwCommand::GetLinks { resp } => {
                        let mut links: Vec<LinkInfo> = s.links.values().copied().collect();
                        links.sort_by_key(|l| l.id);
                        let _ = resp.send(links);
                    }
                    PwCommand::CreateVirtualMic { resp } => {
                        let props = properties!(
                            "factory.name" => "support.null-audio-sink",
                            "node.name" => "pwsp-virtual-mic",
                            "node.description" => "PWSP Virtual Mic",
                            "media.class" => "Audio/Source/Virtual",
                            "audio.position" => "[ FL FR ]",
                            "audio.channels" => "2",
                            "object.linger" => "false",
                        );
                        match core.create_object::<pipewire::node::Node>("adapter", &props) {
                            Ok(node) => {
                                s.proxy_id_counter += 1;
                                let id = s.proxy_id_counter;
                                s.proxies.insert(id, Box::new(node));
                                let _ = resp.send(Ok(id));
                            }
                            Err(e) => {
                                let _ = resp.send(Err(e.to_string()));
                            }
                        }
                    }
                    PwCommand::CreateLink {
                        output_fl,
                        output_fr,
                        input_fl,
                        input_fr,
                        resp,
                    } => {
                        let props_fl = properties! {
                            "link.output.node" => format!("{}", output_fl.node_id).as_str(),
                            "link.output.port" => format!("{}", output_fl.port_id).as_str(),
                            "link.input.node"  => format!("{}", input_fl.node_id).as_str(),
                            "link.input.port"  => format!("{}", input_fl.port_id).as_str(),
                        };
                        let props_fr = properties! {
                            "link.output.node" => format!("{}", output_fr.node_id).as_str(),
                            "link.output.port" => format!("{}", output_fr.port_id).as_str(),
                            "link.input.node"  => format!("{}", input_fr.node_id).as_str(),
                            "link.input.port"  => format!("{}", input_fr.port_id).as_str(),
                        };

                        let link_fl = match core.create_object::<Link>("link-factory", &props_fl) {
                            Ok(link) => link,
                            Err(e) => {
                                let _ = resp.send(Err(e.to_string()));
                                return;
                            }
                        };
                        let link_fr = match core.create_object::<Link>("link-factory", &props_fr) {
                            Ok(link) => link,
                            Err(e) => {
                                let _ = resp.send(Err(e.to_string()));
                                return;
                            }
                        };

                        s.proxy_id_counter += 1;
                        let id_fl = s.proxy_id_counter;
                        s.proxies.insert(id_fl, Box::new(link_fl));

                        s.proxy_id_counter += 1;
                        let id_fr = s.proxy_id_counter;
                        s.proxies.insert(id_fr, Box::new(link_fr));

                        let _ = resp.send(Ok((id_fl, id_fr)));
                    }
                    PwCommand::DestroyObject { id } => {
                        s.proxies.remove(&id);
                    }
                    PwCommand::DestroyGlobal { id } => {
                        s.links.remove(&id);
                        registry.destroy_global(id);
                    }
                }
            });

            main_loop.run();
        });

        // Wait for the pipewire thread to be fully up and processed initial events
        let _ = ready_rx.recv();

        PipewireManager { sender: pw_sender }
    })
}

pub fn setup_pipewire_context() -> Result<(MainLoopRc, ContextRc), String> {
    pipewire::init();
    let main_loop = MainLoopRc::new(None).map_err(|e| e.to_string())?;
    let context = ContextRc::new(&main_loop, None).map_err(|e| e.to_string())?;
    Ok((main_loop, context))
}

enum ParsedGlobal {
    Device(AudioDevice),
    Port(Port),
    Link(LinkInfo),
    Unknown,
}

fn parse_global_object(global_object: &GlobalObject<&DictRef>) -> ParsedGlobal {
    let props = match global_object.props {
        Some(p) => p,
        None => return ParsedGlobal::Unknown,
    };

    if let Some(media_class) = props.get("media.class") {
        let node_id = global_object.id;
        let node_nick = props.get("node.nick");
        let node_name = props.get("node.name");
        let node_description = props.get("node.description");

        // `Audio/Source/Virtual` (our own virtual mic) also matches "Audio/Source",
        // which is intended — it is a microphone as far as the rest of PWSP cares.
        let device_type = if media_class.starts_with("Audio/Source") {
            DeviceType::Input
        } else if media_class.starts_with("Stream/Output/Audio") {
            DeviceType::Output
        } else if media_class.starts_with("Audio/Sink") {
            DeviceType::Sink
        } else {
            return ParsedGlobal::Unknown;
        };

        return ParsedGlobal::Device(AudioDevice::new(
            node_id,
            node_nick,
            node_description,
            node_name,
            device_type,
        ));
    }

    if let (Some(output_node), Some(input_node)) = (
        props
            .get("link.output.node")
            .and_then(|id| id.parse::<u32>().ok()),
        props
            .get("link.input.node")
            .and_then(|id| id.parse::<u32>().ok()),
    ) {
        return ParsedGlobal::Link(LinkInfo {
            id: global_object.id,
            output_node,
            input_node,
        });
    }

    if props.get("port.direction").is_some()
        && let (Some(node_id), Some(port_id), Some(port_name)) = (
            props.get("node.id").and_then(|id| id.parse::<u32>().ok()),
            props.get("port.id").and_then(|id| id.parse::<u32>().ok()),
            props.get("port.name"),
        )
    {
        return ParsedGlobal::Port(Port {
            node_id,
            port_id,
            name: port_name.to_string(),
        });
    }

    ParsedGlobal::Unknown
}

pub async fn get_all_devices() -> Result<AllDevices> {
    let (tx, rx) = oneshot::channel();
    let manager = get_manager();
    manager
        .sender
        .send(PwCommand::GetDevices { resp: tx })
        .map_err(|_| anyhow!("Failed to send GetDevices to manager"))?;
    let res = rx
        .await
        .map_err(|e| anyhow!("Failed to receive response: {}", e))?;
    Ok(res)
}

pub async fn get_all_links() -> Result<Vec<LinkInfo>> {
    let (tx, rx) = oneshot::channel();
    let manager = get_manager();
    manager
        .sender
        .send(PwCommand::GetLinks { resp: tx })
        .map_err(|_| anyhow!("Failed to send GetLinks to manager"))?;
    rx.await
        .map_err(|e| anyhow!("Failed to receive response: {}", e))
}

fn matches_device_name(device: &AudioDevice, device_name: &str) -> bool {
    device.name == device_name
        || device.nick == device_name
        || device.name.contains(device_name)
        || device.nick.contains(device_name)
}

pub async fn get_device(device_name: &str) -> Result<AudioDevice> {
    let devices = get_all_devices().await?;

    devices
        .iter()
        .find(|device| matches_device_name(device, device_name))
        .cloned()
        .ok_or_else(|| anyhow!("Device not found: {}", device_name))
}

/// Re-reads a node by its PipeWire id, so callers always link against fresh ports.
pub async fn get_device_by_id(id: u32) -> Result<AudioDevice> {
    get_all_devices()
        .await?
        .iter()
        .find(|device| device.id == id)
        .cloned()
        .ok_or_else(|| anyhow!("Node {} is gone", id))
}

/// Looks up an `Audio/Sink` node by name.
pub async fn get_sink(name: &str) -> Result<AudioDevice> {
    get_all_devices()
        .await?
        .sinks
        .into_iter()
        .find(|d| matches_device_name(d, name))
        .ok_or_else(|| anyhow!("Output device not found: {}", name))
}

pub struct PwTerminator {
    ids: Vec<u32>,
}

impl Drop for PwTerminator {
    fn drop(&mut self) {
        let manager = get_manager();
        for id in &self.ids {
            let _ = manager.sender.send(PwCommand::DestroyObject { id: *id });
        }
    }
}

pub async fn create_virtual_mic() -> Result<PwTerminator> {
    let (tx, rx) = oneshot::channel();
    let manager = get_manager();
    manager
        .sender
        .send(PwCommand::CreateVirtualMic { resp: tx })
        .map_err(|_| anyhow!("Failed to send CreateVirtualMic to manager"))?;

    let res = rx
        .await
        .map_err(|e| anyhow!("Failed to receive response: {}", e))?;

    let id = res.map_err(|e| anyhow!(e))?;
    Ok(PwTerminator { ids: vec![id] })
}

/// Makes `source_node` feed `target` and nothing else.
///
/// Idempotent: safe to call on every device-check tick. Returns `Some` only when a new link
/// was created, so an already-correct route is left in place and the caller never tears
/// down a link it does not own.
///
/// The link is created *before* stale ones are pruned. That order matters: the node is
/// never left unlinked, which is the state that would invite the session manager to
/// re-attach it somewhere else.
pub async fn ensure_route(
    source_node: &AudioDevice,
    target: &AudioDevice,
) -> Result<Option<PwTerminator>> {
    let existing = get_all_links().await?;
    let already_routed = existing
        .iter()
        .any(|link| link.output_node == source_node.id && link.input_node == target.id);

    let terminator = if already_routed {
        None
    } else {
        let output_fl = source_node
            .output_fl
            .clone()
            .ok_or_else(|| anyhow!("Node {} has no output_FL port", source_node.name))?;
        let output_fr = source_node
            .output_fr
            .clone()
            .ok_or_else(|| anyhow!("Node {} has no output_FR port", source_node.name))?;
        let input_fl = target
            .input_fl
            .clone()
            .ok_or_else(|| anyhow!("Node {} has no input_FL port", target.name))?;
        let input_fr = target
            .input_fr
            .clone()
            .ok_or_else(|| anyhow!("Node {} has no input_FR port", target.name))?;

        Some(create_link(output_fl, output_fr, input_fl, input_fr).await?)
    };

    prune_links_from(source_node.id, target.id).await?;

    Ok(terminator)
}

/// Destroys every link leaving `source_node` that does not end at `keep_target`.
async fn prune_links_from(source_node: u32, keep_target: u32) -> Result<()> {
    let manager = get_manager();
    for link in get_all_links().await? {
        if link.output_node == source_node && link.input_node != keep_target {
            let _ = manager
                .sender
                .send(PwCommand::DestroyGlobal { id: link.id });
        }
    }
    Ok(())
}

pub async fn create_link(
    output_fl: Port,
    output_fr: Port,
    input_fl: Port,
    input_fr: Port,
) -> Result<PwTerminator> {
    let (tx, rx) = oneshot::channel();
    let manager = get_manager();
    manager
        .sender
        .send(PwCommand::CreateLink {
            output_fl,
            output_fr,
            input_fl,
            input_fr,
            resp: tx,
        })
        .map_err(|_| anyhow!("Failed to send CreateLink to manager"))?;

    let res = rx
        .await
        .map_err(|e| anyhow!("Failed to receive response: {}", e))?;

    let (id_fl, id_fr) = res.map_err(|e| anyhow!(e))?;
    Ok(PwTerminator {
        ids: vec![id_fl, id_fr],
    })
}
