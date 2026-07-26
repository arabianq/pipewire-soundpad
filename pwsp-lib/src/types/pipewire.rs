#[derive(Debug)]
pub struct Terminate {}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Port {
    pub node_id: u32,
    pub port_id: u32,

    pub name: String,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DeviceType {
    Input,
    Output,
    Sink,
}

/// A link between two nodes in the PipeWire graph.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct LinkInfo {
    pub id: u32,
    pub output_node: u32,
    pub input_node: u32,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct AudioDevice {
    pub id: u32,

    pub nick: String,
    pub name: String,

    pub device_type: DeviceType,

    pub input_fl: Option<Port>,
    pub input_fr: Option<Port>,
    pub output_fl: Option<Port>,
    pub output_fr: Option<Port>,
}

impl AudioDevice {
    pub fn new(
        id: u32,
        nick: Option<&str>,
        description: Option<&str>,
        name: Option<&str>,
        device_type: DeviceType,
    ) -> Self {
        Self {
            id,
            nick: nick
                .or(description)
                .or(name)
                .unwrap_or_default()
                .to_string(),
            name: name.unwrap_or_default().to_string(),
            device_type,
            input_fl: None,
            input_fr: None,
            output_fl: None,
            output_fr: None,
        }
    }

    pub fn add_port(&mut self, port: Port) {
        // `playback_*` are the input ports of an `Audio/Sink` node. `monitor_*` are its
        // output ports and are deliberately left unmapped — we never record from a sink.
        match port.name.as_str() {
            "input_FL" | "playback_FL" => self.input_fl = Some(port),
            "input_FR" | "playback_FR" => self.input_fr = Some(port),
            "output_FL" | "capture_FL" => self.output_fl = Some(port),
            "output_FR" | "capture_FR" => self.output_fr = Some(port),
            "input_MONO" | "playback_MONO" => {
                self.input_fl = Some(port.clone());
                self.input_fr = Some(port);
            }
            "output_MONO" | "capture_MONO" => {
                self.output_fl = Some(port.clone());
                self.output_fr = Some(port);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_device_new() {
        let device = AudioDevice::new(
            1,
            Some("NickName"),
            Some("Description"),
            Some("Name"),
            DeviceType::Input,
        );
        assert_eq!(device.id, 1);
        assert_eq!(device.nick, "NickName");
        assert_eq!(device.name, "Name");
        assert_eq!(device.device_type, DeviceType::Input);

        // Fallbacks for nick
        let device_no_nick =
            AudioDevice::new(2, None, Some("Desc"), Some("Name"), DeviceType::Output);
        assert_eq!(device_no_nick.nick, "Desc");

        let device_no_desc = AudioDevice::new(3, None, None, Some("Name"), DeviceType::Output);
        assert_eq!(device_no_desc.nick, "Name");
    }

    #[test]
    fn test_audio_device_sink_ports() {
        let mut sink = AudioDevice::new(1, None, None, Some("speakers"), DeviceType::Sink);

        let port = |id: u32, name: &str| Port {
            node_id: 1,
            port_id: id,
            name: name.to_string(),
        };

        // playback_* carries audio into the sink, so those are its inputs.
        sink.add_port(port(10, "playback_FL"));
        sink.add_port(port(11, "playback_FR"));
        assert_eq!(sink.input_fl, Some(port(10, "playback_FL")));
        assert_eq!(sink.input_fr, Some(port(11, "playback_FR")));

        // monitor_* is what the sink plays out; linking a stream there would be a loop.
        sink.add_port(port(12, "monitor_FL"));
        sink.add_port(port(13, "monitor_FR"));
        assert_eq!(sink.output_fl, None);
        assert_eq!(sink.output_fr, None);
    }

    #[test]
    fn test_audio_device_add_port() {
        let mut device = AudioDevice::new(1, None, None, Some("device-name"), DeviceType::Input);

        let port_fl = Port {
            node_id: 1,
            port_id: 10,
            name: "input_FL".to_string(),
        };
        let port_fr = Port {
            node_id: 1,
            port_id: 11,
            name: "input_FR".to_string(),
        };

        device.add_port(port_fl.clone());
        device.add_port(port_fr.clone());

        assert_eq!(device.input_fl, Some(port_fl));
        assert_eq!(device.input_fr, Some(port_fr));

        // Test output ports
        let port_out_fl = Port {
            node_id: 1,
            port_id: 12,
            name: "output_FL".to_string(),
        };
        let port_out_fr = Port {
            node_id: 1,
            port_id: 13,
            name: "capture_FR".to_string(),
        };

        device.add_port(port_out_fl.clone());
        device.add_port(port_out_fr.clone());

        assert_eq!(device.output_fl, Some(port_out_fl));
        assert_eq!(device.output_fr, Some(port_out_fr));

        // Test MONO ports
        let mut device_mono =
            AudioDevice::new(2, None, None, Some("mono-device"), DeviceType::Input);
        let port_mono = Port {
            node_id: 2,
            port_id: 20,
            name: "input_MONO".to_string(),
        };
        device_mono.add_port(port_mono.clone());

        assert_eq!(device_mono.input_fl, Some(port_mono.clone()));
        assert_eq!(device_mono.input_fr, Some(port_mono));
    }
}
