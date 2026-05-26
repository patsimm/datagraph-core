use std::{collections::HashMap, fmt::Display, hash::Hash, ops::Index, str::FromStr};

use crate::graph::{GraphError, NodeId, PortType};

pub trait SampleProcessor {
    fn process_sample(&mut self, inputs: &[f32], outputs: &mut [f32]);
}

pub trait Tickable {
    fn tick(&mut self);
}

pub trait BatchTickable {
    fn tick_batch(&mut self, outputs: &mut HashMap<PortKey, &mut [f32]>);
}

pub trait PortValueAccess {
    fn port_value(&self, node_id: NodeId, port: usize, port_type: PortType) -> Option<&f32>;
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
            node_id: node_id.into(),
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

pub struct PortValueBuffer<'a, 'b> {
    buffer: &'a mut HashMap<PortKey, &'b mut [f32]>,
}

impl<'a, 'b> Index<PortKey> for PortValueBuffer<'a, 'b> {
    type Output = [f32];
    fn index(&self, port_id: PortKey) -> &Self::Output {
        self.buffer[&port_id]
    }
}

impl<'a, 'b> PortValueBuffer<'a, 'b> {
    fn new(buffer: &'a mut HashMap<PortKey, &'b mut [f32]>) -> Self {
        PortValueBuffer { buffer }
    }
    fn buffer_sample(
        &mut self,
        index: usize,
        port_value_accessor: &dyn PortValueAccess,
    ) -> Result<(), String> {
        for (key, buffer) in self.buffer.iter_mut() {
            if let Some(value) =
                port_value_accessor.port_value(key.node_id, key.port_index, key.port_type)
            {
                buffer[index] = *value;
            } else {
                return Err(format!("Port value not found for {:?}", key));
            }
        }
        Ok(())
    }
}

impl<T: Tickable + PortValueAccess> BatchTickable for T {
    fn tick_batch(&mut self, outputs: &mut HashMap<PortKey, &mut [f32]>) {
        let batchsize = outputs
            .values()
            .next()
            .map(|buffer| buffer.len())
            .expect("Output buffers cannot be empty");
        let mut buffer = PortValueBuffer::new(outputs);
        for i in 0..batchsize {
            self.tick();
            buffer
                .buffer_sample(i, self)
                .expect("Failed to buffer sample");
        }
    }
}
