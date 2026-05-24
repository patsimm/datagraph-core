use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

pub struct PortInfo {
    pub port_index: usize,
    pub port_type: PortType,
    pub name: &'static str,
}
