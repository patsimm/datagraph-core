use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::graph::{GraphError, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum PortType {
    #[serde(rename = "in")]
    Input,
    #[serde(rename = "out")]
    Output,
}

#[derive(Clone, Debug, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    #[tsify(type = "string")]
    portkey: PortKey,
    name: &'static str,
}

impl PortInfo {
    pub fn new(
        node_id: NodeId,
        port_index: usize,
        port_type: PortType,
        name: &'static str,
    ) -> Self {
        Self {
            portkey: PortKey::new(node_id, port_index, port_type),
            name,
        }
    }

    pub fn node_id(&self) -> &NodeId {
        &self.portkey.node_id
    }

    pub fn port_index(&self) -> usize {
        self.portkey.port_index
    }

    pub fn port_type(&self) -> PortType {
        self.portkey.port_type
    }

    pub fn key(&self) -> &PortKey {
        &self.portkey
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PortKey {
    node_id: NodeId,
    port_index: usize,
    port_type: PortType,
}

impl serde::Serialize for PortKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for PortKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        PortKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl PortKey {
    pub fn new(node_id: NodeId, port_index: usize, port_type: PortType) -> Self {
        Self {
            node_id,
            port_index,
            port_type,
        }
    }

    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn port_index(&self) -> usize {
        self.port_index
    }

    pub fn port_type(&self) -> PortType {
        self.port_type
    }
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
            node_id: node_id
                .parse()
                .map_err(|_| GraphError::InvalidPortKey { key: s.into() })?,
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
