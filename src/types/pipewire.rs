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
