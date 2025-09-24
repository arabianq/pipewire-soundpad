use crate::types::pipewire::{AudioDevice, DeviceType, Port, Terminate};
use pipewire::{
    context::Context, link::Link, main_loop::MainLoop, properties::properties,
    registry::GlobalObject, spa::utils::dict::DictRef,
};
use std::{collections::HashMap, error::Error, thread};
use tokio::{
    sync::mpsc,
    time::{Duration, timeout},
};

fn parse_global_object(
    global_object: &GlobalObject<&DictRef>,
) -> (Option<AudioDevice>, Option<Port>) {
    // Only objects with props can be devices/ports
    if let Some(props) = global_object.props {
        // Only objects with media.class can be devices
        if let Some(media_class) = props.get("media.class") {
            let node_id = global_object.id;
            let node_nick = props.get("node.nick");
            let node_name = props.get("node.name");
            let node_description = props.get("node.description");

            // Check if the device is an input or output
            return if media_class.starts_with("Audio/Source") {
                let input_device = AudioDevice {
                    id: node_id,
                    nick: node_nick
                        .unwrap_or(node_description.unwrap_or(node_name.unwrap_or_default()))
                        .to_string(),
                    name: node_name.unwrap_or_default().to_string(),
                    device_type: DeviceType::Input,

                    input_fl: None,
                    input_fr: None,
                    output_fl: None,
                    output_fr: None,
                };
                (Some(input_device), None)
            } else if media_class.starts_with("Stream/Output/Audio") {
                let output_device = AudioDevice {
                    id: node_id,
                    nick: node_nick
                        .unwrap_or(node_description.unwrap_or(node_name.unwrap_or_default()))
                        .to_string(),
                    name: node_name.unwrap_or_default().to_string(),
                    device_type: DeviceType::Output,

                    input_fl: None,
                    input_fr: None,
                    output_fl: None,
                    output_fr: None,
                };
                (Some(output_device), None)
            } else {
                (None, None)
            };
            // Check if the object is a port
        } else if props.get("port.direction").is_some() {
            let node_id = props.get("node.id").unwrap().parse::<u32>().unwrap();
            let port_id = props.get("port.id").unwrap().parse::<u32>().unwrap();
            let port_name = props.get("port.name").unwrap();

            let port = Port {
                node_id,
                port_id,
                name: port_name.to_string(),
            };

            return (None, Some(port));
        }
    }
    (None, None)
}

async fn pw_get_global_objects_thread(
    main_sender: mpsc::Sender<(Option<AudioDevice>, Option<Port>)>,
    pw_receiver: pipewire::channel::Receiver<Terminate>,
) {
    let main_loop = MainLoop::new(None).expect("Failed to initialize pipewire main loop");

    // Stop main loop on Terminate message
    let _receiver = pw_receiver.attach(main_loop.loop_(), {
        let _main_loop = main_loop.clone();
        move |_| _main_loop.quit()
    });

    let context = Context::new(&main_loop).expect("Failed to create pipewire context");
    let core = context
        .connect(None)
        .expect("Failed to connect to pipewire context");
    let registry = core
        .get_registry()
        .expect("Failed to get registry from pipewire context");

    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            // Try to parse every global object pipewire finds
            let (device, port) = parse_global_object(global);

            // Send message to the main thread
            let sender_clone = main_sender.clone();
            tokio::task::spawn(async move {
                sender_clone.send((device, port)).await.ok();
            });
        })
        .register();

    main_loop.run();
}

pub async fn get_all_devices() -> Result<(Vec<AudioDevice>, Vec<AudioDevice>), Box<dyn Error>> {
    // Channels to communicate with pipewire thread
    let (main_sender, mut main_receiver) = mpsc::channel(10);
    let (pw_sender, pw_receiver) = pipewire::channel::channel();

    // Spawn pipewire thread in background
    let _pw_thread =
        tokio::spawn(async move { pw_get_global_objects_thread(main_sender, pw_receiver).await });

    let mut input_devices: HashMap<u32, AudioDevice> = HashMap::new();
    let mut output_devices: HashMap<u32, AudioDevice> = HashMap::new();
    let mut ports: Vec<Port> = vec![];

    loop {
        // If we don't receive a message in 100ms, we can assume that pipewire thread is finished
        match timeout(Duration::from_millis(100), main_receiver.recv()).await {
            Ok(Some((device, port))) => {
                if let Some(device) = device {
                    match device.device_type {
                        DeviceType::Input => {
                            input_devices.insert(device.id, device);
                        }
                        DeviceType::Output => {
                            output_devices.insert(device.id, device);
                        }
                    }
                } else if let Some(port) = port {
                    ports.push(port);
                }
            }
            Ok(None) | Err(_) => {
                // Pipewire thread is finished and we can collect our devices
                pw_sender
                    .send(Terminate {})
                    .expect("Failed to terminate pipewire thread");

                for port in ports {
                    let node_id = port.node_id;

                    if input_devices.contains_key(&node_id) {
                        let input_device = input_devices.get_mut(&node_id).unwrap();
                        match port.name.as_str() {
                            "input_FL" => input_device.input_fl = Some(port),
                            "input_FR" => input_device.input_fr = Some(port),
                            "output_FL" => input_device.output_fl = Some(port),
                            "output_FR" => input_device.output_fr = Some(port),
                            "capture_FL" => input_device.output_fl = Some(port),
                            "capture_FR" => input_device.output_fr = Some(port),
                            "input_MONO" => {
                                input_device.input_fl = Some(port.clone());
                                input_device.input_fr = Some(port)
                            }
                            _ => {}
                        }
                    } else if output_devices.contains_key(&node_id) {
                        let output_device = output_devices.get_mut(&node_id).unwrap();
                        match port.name.as_str() {
                            "input_FL" => output_device.input_fl = Some(port),
                            "input_FR" => output_device.input_fr = Some(port),
                            "output_FL" => output_device.output_fl = Some(port),
                            "output_FR" => output_device.output_fr = Some(port),
                            "capture_FL" => output_device.output_fl = Some(port),
                            "capture_FR" => output_device.output_fr = Some(port),
                            "output_MONO" => {
                                output_device.output_fl = Some(port.clone());
                                output_device.output_fr = Some(port)
                            }
                            "capture_MONO" => {
                                output_device.output_fl = Some(port.clone());
                                output_device.output_fr = Some(port)
                            }
                            _ => {}
                        }
                    }
                }

                let mut input_devices: Vec<AudioDevice> = input_devices.values().cloned().collect();
                let mut output_devices: Vec<AudioDevice> =
                    output_devices.values().cloned().collect();

                input_devices.sort_by(|a, b| a.id.cmp(&b.id));
                output_devices.sort_by(|a, b| a.id.cmp(&b.id));

                return Ok((input_devices, output_devices));
            }
        }
    }
}

pub async fn get_device(node_id: u32) -> Result<AudioDevice, Box<dyn Error>> {
    let (mut input_devices, output_devices) = get_all_devices().await?;
    input_devices.extend(output_devices);

    for device in input_devices {
        if device.id == node_id {
            return Ok(device);
        }
    }

    Err("Device not found".into())
}

pub fn create_virtual_mic() -> Result<pipewire::channel::Sender<Terminate>, Box<dyn Error>> {
    let (pw_sender, pw_receiver) = pipewire::channel::channel::<Terminate>();

    let _pw_thread = thread::spawn(move || {
        let main_loop = MainLoop::new(None).expect("Failed to initialize pipewire main loop");
        let context = Context::new(&main_loop).expect("Failed to create pipewire context");
        let core = context
            .connect(None)
            .expect("Failed to connect to pipewire context");

        let props = properties!(
            "factory.name" => "support.null-audio-sink",
            "node.name" => "pwsp-virtual-mic",
            "node.description" => "PWSP Virtual Mic",
            "media.class" => "Audio/Source/Virtual",
            "audio.position" => "[ FL FR ]",
            "audio.channels" => "2",
            "object.linger" => "false", // Destroy the node on app exit
        );

        let _node = core
            .create_object::<pipewire::node::Node>("adapter", &props)
            .expect("Failed to create virtual mic");

        let _receiver = pw_receiver.attach(main_loop.loop_(), {
            let _main_loop = main_loop.clone();
            move |_| _main_loop.quit()
        });

        println!("Virtual mic created");
        main_loop.run();
    });

    Ok(pw_sender)
}

pub fn create_link(
    output_fl: Port,
    output_fr: Port,
    input_fl: Port,
    input_fr: Port,
) -> Result<pipewire::channel::Sender<Terminate>, Box<dyn Error>> {
    let (pw_sender, pw_receiver) = pipewire::channel::channel::<Terminate>();

    let _pw_thread = thread::spawn(move || {
        let main_loop = MainLoop::new(None).expect("Failed to initialize pipewire main loop");
        let context = Context::new(&main_loop).expect("Failed to create pipewire context");
        let core = context
            .connect(None)
            .expect("Failed to connect to pipewire context");

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

        let _link_fl = core
            .create_object::<Link>("link-factory", &props_fl)
            .expect("Failed to create link FL");
        let _link_fr = core
            .create_object::<Link>("link-factory", &props_fr)
            .expect("Failed to create link FR");

        let _receiver = pw_receiver.attach(main_loop.loop_(), {
            let _main_loop = main_loop.clone();
            move |_| _main_loop.quit()
        });

        println!(
            "Link created: FL: {}-{} FR: {}-{}",
            output_fl.node_id, input_fl.node_id, output_fr.node_id, input_fr.node_id
        );
        main_loop.run();
    });

    Ok(pw_sender)
}
