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
        match port.name.as_str() {
            "input_FL" => self.input_fl = Some(port),
            "input_FR" => self.input_fr = Some(port),
            "output_FL" | "capture_FL" => self.output_fl = Some(port),
            "output_FR" | "capture_FR" => self.output_fr = Some(port),
            "input_MONO" => {
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
