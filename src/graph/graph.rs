use log::info;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::batch::BatchBuffer;
use super::connection::Connection;
use super::error::GraphError;
use super::node::{CreateNode, GraphNode, NodeInfo};
use super::node_id::NodeId;
use super::param::{Param, ParamHandle};
use super::port::{PortInfo, PortKey, PortType};

pub trait PortValueAccess {
    fn port_value(&self, node_id: NodeId, port: usize, port_type: PortType) -> Option<&f32>;
}

#[wasm_bindgen]
pub struct Graph {
    nodes: HashMap<NodeId, GraphNode>,
    param_handles: HashMap<NodeId, ParamHandle>,
    connections: Vec<Connection>,
    sample_rate: u32,
    batch_buffer: BatchBuffer,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            param_handles: HashMap::new(),
            connections: Vec::new(),
            sample_rate: 44100,
            batch_buffer: BatchBuffer::default(),
        }
    }
}

impl Graph {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    pub fn add<T: CreateNode>(&mut self) -> NodeId {
        let graph_node = T::create(self.sample_rate);
        self.add_node(graph_node)
    }

    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let id = NodeId::new();
        self.nodes.insert(id, node);
        id
    }

    pub fn add_param(&mut self, value: f32) -> NodeId {
        let param = Param::from(value);
        let param_handle = param.handle();
        let graph_node = GraphNode::new(param);
        let id = self.add_node(graph_node);
        self.param_handles.insert(id, param_handle);
        id
    }

    pub fn set_param_value(&mut self, node_id: NodeId, value: f32) -> Result<(), GraphError> {
        self.node_exists(node_id)?;
        if let Some(handle) = self.param_handles.get_mut(&node_id) {
            handle.set(value);
            Ok(())
        } else {
            Err(GraphError::NotAParameter { node_id })
        }
    }

    pub fn remove_node(&mut self, node: NodeId) -> Result<(), GraphError> {
        self.node_exists(node)?;
        self.nodes.remove(&node);
        self.connections
            .retain(|conn| conn.from != node && conn.to != node);
        Ok(())
    }

    pub fn info(&self, node: NodeId) -> Result<NodeInfo, GraphError> {
        self.nodes
            .get(&node)
            .map(|node| node.node_info())
            .ok_or(GraphError::NodeNotFound { node_id: node })
    }

    pub fn connect(
        &mut self,
        from: NodeId,
        from_port: usize,
        to: NodeId,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.port_exists(from, from_port, PortType::Output)?;
        if from == to {
            return Err(GraphError::ImpossibleConnection {
                from_node_id: from,
                from_port,
                to_node_id: to,
                to_port,
            });
        }
        self.port_is_free(to, to_port)?;

        self.connections.push(Connection {
            from,
            from_port,
            to,
            to_port,
        });

        Ok(())
    }

    pub fn disconnect(
        &mut self,
        from: NodeId,
        from_port: usize,
        to: NodeId,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.port_exists(from, from_port, PortType::Output)?;
        self.port_exists(to, to_port, PortType::Input)?;

        self.connections.retain(|conn| {
            !(conn.from == from
                && conn.from_port == from_port
                && conn.to == to
                && conn.to_port == to_port)
        });

        Ok(())
    }

    pub fn port_info(&self, node_id: NodeId, port: usize, port_type: PortType) -> Option<PortInfo> {
        let node = self.nodes.get(&node_id)?;
        node.port_info(port_type, port)
    }

    pub fn set_default_input_value(
        &mut self,
        node_id: NodeId,
        port: usize,
        value: f32,
    ) -> Result<(), GraphError> {
        self.port_exists(node_id, port, PortType::Input)?;
        self.nodes
            .get_mut(&node_id)
            .expect("node exists — checked by port_exists")
            .set_default_input_value(port, value);
        Ok(())
    }

    fn node_exists(&self, node_id: NodeId) -> Result<(), GraphError> {
        self.nodes
            .get(&node_id)
            .ok_or(GraphError::NodeNotFound { node_id })?;
        Ok(())
    }

    fn port_exists(
        &self,
        node_id: NodeId,
        port: usize,
        port_type: PortType,
    ) -> Result<(), GraphError> {
        let node = self
            .nodes
            .get(&node_id)
            .ok_or(GraphError::NodeNotFound { node_id })?;
        node.port_info(port_type, port).map_or(
            Err(GraphError::PortNotFound {
                node_id,
                port,
                port_type,
            }),
            |_| Ok(()),
        )
    }

    fn port_is_free(&self, node_id: NodeId, port: usize) -> Result<(), GraphError> {
        self.port_exists(node_id, port, PortType::Input)?;
        for conn in &self.connections {
            if conn.to == node_id && conn.to_port == port {
                return Err(GraphError::PortAlreadyConnected {
                    node_id,
                    port,
                    port_type: PortType::Input,
                });
            }
        }
        Ok(())
    }
}

pub trait Tickable {
    fn tick(&mut self);
}

impl Tickable for Graph {
    fn tick(&mut self) {
        for node in self.nodes.values_mut() {
            node.reset_input_cache();
        }
        for conn in &self.connections {
            let value = *self.nodes[&conn.from].output_value(conn.from_port);
            self.nodes
                .get_mut(&conn.to)
                .unwrap()
                .set_input_value(conn.to_port, value);
        }
        self.nodes.iter_mut().for_each(|(_, node)| node.tick());
    }
}

pub trait BatchTickable {
    fn tick_batch<'a>(
        &'a mut self,
        outputs: &[PortKey],
        batchsize: usize,
    ) -> impl Iterator<Item = &'a [f32]>;
}

impl BatchTickable for Graph {
    fn tick_batch<'a>(
        &'a mut self,
        output_ports: &[PortKey],
        batchsize: usize,
    ) -> impl Iterator<Item = &'a [f32]> {
        self.batch_buffer.resize(output_ports, batchsize);
        for sample_index in 0..batchsize {
            self.tick();
            for (port_index, port) in output_ports.iter().enumerate() {
                let val = *self.nodes[&port.node_id].output_value(port.port_index);
                self.batch_buffer.set_value(port_index, sample_index, val);
            }
        }
        (&self.batch_buffer).into_iter()
    }
}

impl PortValueAccess for Graph {
    fn port_value(&self, node_id: NodeId, port: usize, port_type: PortType) -> Option<&f32> {
        self.nodes
            .get(&node_id)
            .and_then(|node| node.port_value(port_type, port))
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use super::{Graph, PortValueAccess, Tickable};
    use crate::graph::{Node, Param, PortType};
    use crate::nodes::Add;

    #[test]
    fn constant_outputs_constant() {
        let mut param: Param = 0.5.into();
        let mut output = [0.0; 1];
        param.process([], &mut output);
        assert_eq!(output, [0.5]);
    }

    #[test]
    fn graph_adds_nodes() {
        let mut graph = Graph::new(1);
        let constant_id1 = graph.add_param(0.5);
        assert_eq!(
            graph.info(constant_id1).unwrap().node_type(),
            type_name::<Param>()
        );
    }

    #[test]
    fn graph_connects_nodes() {
        let mut graph = Graph::new(1);
        let constant_id = graph.add_param(0.5);
        let constant_id2 = graph.add_param(0.5);
        let adder_id = graph.add::<Add>();
        graph
            .connect(constant_id, 0, adder_id, 0)
            .expect("Failed to connect nodes");
        graph
            .connect(constant_id2, 0, adder_id, 1)
            .expect("Failed to connect nodes");
        graph.tick();
        let output = graph.port_value(adder_id, 0, PortType::Output).unwrap();
        assert_eq!(*output, 1.0);
        let input1 = graph.port_value(adder_id, 0, PortType::Input).unwrap();
        assert_eq!(*input1, 0.5);
    }

    #[test]
    fn graph_connect_fails_when_invalid_port() {
        let mut graph = Graph::new(1);
        let constant_id = graph.add_param(0.5);
        let adder_id = graph.add::<Add>();
        let result = graph.connect(constant_id, 1, adder_id, 0);
        assert!(result.is_err());
    }
}
