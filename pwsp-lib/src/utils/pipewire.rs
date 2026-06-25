use crate::types::pipewire::{AudioDevice, DeviceType, Port};
use anyhow::{Result, anyhow};
use pipewire::{
    context::ContextRc, link::Link, main_loop::MainLoopRc, properties::properties,
    registry::GlobalObject, spa::utils::dict::DictRef,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::OnceLock, thread};
use tokio::sync::oneshot;

pub enum PwCommand {
    GetDevices {
        resp: oneshot::Sender<(Vec<AudioDevice>, Vec<AudioDevice>)>,
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
    DestroyObject {
        id: u32,
    },
}

struct AppState {
    input_devices: HashMap<u32, AudioDevice>,
    output_devices: HashMap<u32, AudioDevice>,
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

            // Leak to fix lifetime issues since this thread lives forever
            let core = Box::leak(Box::new(
                context
                    .connect(None)
                    .expect("Failed to connect to pipewire"),
            ));
            let registry = Box::leak(Box::new(
                core.get_registry().expect("Failed to get registry"),
            ));

            let state = Rc::new(RefCell::new(AppState {
                input_devices: HashMap::new(),
                output_devices: HashMap::new(),
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
                    let (device, port) = parse_global_object(global);
                    let mut s = state_for_registry_add.borrow_mut();
                    if let Some(device) = device {
                        match device.device_type {
                            DeviceType::Input => {
                                s.input_devices.insert(device.id, device);
                            }
                            DeviceType::Output => {
                                s.output_devices.insert(device.id, device);
                            }
                        }
                    } else if let Some(port) = port {
                        let node_id = port.node_id;
                        s.ports.insert(port.port_id, port.clone());
                        if let Some(d) = s.input_devices.get_mut(&node_id) {
                            d.add_port(port.clone());
                        } else if let Some(d) = s.output_devices.get_mut(&node_id) {
                            d.add_port(port);
                        }
                    }
                })
                .global_remove(move |id| {
                    let mut s = state_for_registry_remove.borrow_mut();
                    s.input_devices.remove(&id);
                    s.output_devices.remove(&id);
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
                        inputs.sort_by_key(|a| a.id);
                        outputs.sort_by_key(|a| a.id);
                        let _ = resp.send((inputs, outputs));
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

fn parse_global_object(
    global_object: &GlobalObject<&DictRef>,
) -> (Option<AudioDevice>, Option<Port>) {
    let props = match global_object.props {
        Some(p) => p,
        None => return (None, None),
    };

    if let Some(media_class) = props.get("media.class") {
        let node_id = global_object.id;
        let node_nick = props.get("node.nick");
        let node_name = props.get("node.name");
        let node_description = props.get("node.description");

        if media_class.starts_with("Audio/Source") {
            let input_device = AudioDevice::new(
                node_id,
                node_nick,
                node_description,
                node_name,
                DeviceType::Input,
            );
            return (Some(input_device), None);
        } else if media_class.starts_with("Stream/Output/Audio") {
            let output_device = AudioDevice::new(
                node_id,
                node_nick,
                node_description,
                node_name,
                DeviceType::Output,
            );
            return (Some(output_device), None);
        }
        return (None, None);
    }

    if props.get("port.direction").is_some()
        && let (Some(node_id), Some(port_id), Some(port_name)) = (
            props.get("node.id").and_then(|id| id.parse::<u32>().ok()),
            props.get("port.id").and_then(|id| id.parse::<u32>().ok()),
            props.get("port.name"),
        )
    {
        let port = Port {
            node_id,
            port_id,
            name: port_name.to_string(),
        };
        return (None, Some(port));
    }

    (None, None)
}

pub async fn get_all_devices() -> Result<(Vec<AudioDevice>, Vec<AudioDevice>)> {
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

pub async fn get_device(device_name: &str) -> Result<AudioDevice> {
    let (input_devices, output_devices) = get_all_devices().await?;

    input_devices
        .into_iter()
        .chain(output_devices)
        .find(|device| {
            device.name == device_name
                || device.nick == device_name
                || device.name.contains(device_name)
                || device.nick.contains(device_name)
        })
        .ok_or_else(|| anyhow!("Device not found"))
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

pub async fn link_player_to_virtual_mic() -> Result<PwTerminator> {
    let pwsp_daemon_output = match get_device("pwsp-daemon").await {
        Ok(device) => device,
        Err(_) => {
            return Err(anyhow!(
                "Could not find alsa_playback.pwsp-daemon device, skipping device linking"
            ));
        }
    };

    let pwsp_daemon_input = match get_device("pwsp-virtual-mic").await {
        Ok(device) => device,
        Err(_) => {
            return Err(anyhow!(
                "Could not find pwsp-virtual-mic device, skipping device linking"
            ));
        }
    };

    let output_fl = match pwsp_daemon_output.output_fl {
        Some(port) => port,
        None => return Err(anyhow!("Failed to get pwsp-daemon output_fl")),
    };
    let output_fr = match pwsp_daemon_output.output_fr {
        Some(port) => port,
        None => return Err(anyhow!("Failed to get pwsp-daemon output_fr")),
    };
    let input_fl = match pwsp_daemon_input.input_fl {
        Some(port) => port,
        None => return Err(anyhow!("Failed to get pwsp-virtual-mic input_fl")),
    };
    let input_fr = match pwsp_daemon_input.input_fr {
        Some(port) => port,
        None => return Err(anyhow!("Failed to get pwsp-virtual-mic input_fr")),
    };

    create_link(output_fl, output_fr, input_fl, input_fr).await
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
