use std::{fmt::Display, str::FromStr};

use wasm_bindgen::prelude::*;

use crate::graph::{GraphError, NodeId};

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortType {
    Input,
    Output,
}

pub struct PortInfo {
    pub port_index: usize,
    pub port_type: PortType,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortKey {
    pub node_id: NodeId,
    pub port_index: usize,
    pub port_type: PortType,
}

// export function parsePortKey(key: string): PortInfo {
//   const [node, port] = key.split("[");
//   const [portType, portIndex] = port.split("]")[0].split(":");
//   return { nodeId: node, port: parseInt(portIndex), portType: portType as "in" | "out" };
// }
impl FromStr for PortKey {
    type Err = GraphError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('[').collect();
        let node_id = parts
            .first()
            .ok_or_else(|| GraphError::InvalidPortKey { key: s.into() })?
            .trim()
            .to_string();
        let port_part = parts
            .get(1)
            .ok_or_else(|| GraphError::InvalidPortKey { key: s.into() })?
            .split(']')
            .next()
            .ok_or_else(|| GraphError::InvalidPortKey { key: s.into() })?;
        let port_parts: Vec<&str> = port_part.split(':').collect();
        let port_type_str = port_parts
            .first()
            .ok_or_else(|| GraphError::InvalidPortKey { key: s.into() })?
            .trim();
        let port_index_str = port_parts
            .get(1)
            .ok_or_else(|| GraphError::InvalidPortKey { key: s.into() })?
            .trim();
        Ok(PortKey {
            node_id: node_id.parse().map_err(|_| GraphError::InvalidPortKey { key: s.into() })?,
            port_index: port_index_str
                .parse()
                .map_err(|_| GraphError::InvalidPortKey { key: s.into() })?,
            port_type: match port_type_str {
                "in" => PortType::Input,
                "out" => PortType::Output,
                _ => return Err(GraphError::InvalidPortKey { key: s.into() }),
            },
        })
    }
}

impl Display for PortKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let port_type_str = match self.port_type {
            PortType::Input => "in",
            PortType::Output => "out",
        };
        write!(f, "{}[{}:{}]", self.node_id, port_type_str, self.port_index)
    }
}
