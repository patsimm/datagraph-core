use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::batch::BatchBuffer;
use super::connection::Connection;
use super::error::GraphError;
use super::node::{CreateNode, GraphNode, NodeInfo};
use super::node_id::NodeId;
use super::param::{Param, ParamHandle};
use super::port::{PortInfo, PortKey, PortType};
use crate::event_queue::EventSender;

pub trait PortValueAccess {
    fn port_value(&self, port: &PortKey) -> Option<&f32>;
}

#[wasm_bindgen]
pub struct Graph {
    nodes: HashMap<NodeId, GraphNode>,
    param_handles: HashMap<NodeId, ParamHandle>,
    connections: Vec<Connection>,
    sample_rate: u32,
    batch_buffer: BatchBuffer,
    event_sender: Option<EventSender>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            param_handles: HashMap::new(),
            connections: Vec::new(),
            sample_rate: 44100,
            batch_buffer: BatchBuffer::default(),
            event_sender: None,
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

    pub fn new_with_events(sample_rate: u32, event_sender: EventSender) -> Self {
        Self {
            sample_rate,
            event_sender: Some(event_sender),
            ..Default::default()
        }
    }

    pub fn add<T: CreateNode>(&mut self) -> NodeInfo {
        let node_id = NodeId::new();
        self.add_node_with_id::<T>(node_id)
    }

    pub fn add_node_with_id<T: CreateNode>(&mut self, id: NodeId) -> NodeInfo {
        let graph_node = T::create(id, self.sample_rate);
        let node_meta = graph_node.node_meta().clone();
        let default_inputs = graph_node.default_inputs().to_vec();
        self.nodes.insert(id, graph_node);
        NodeInfo::new(id, node_meta, default_inputs)
    }

    pub fn insert_node(&mut self, id: NodeId, node: GraphNode) {
        if let Some(s) = &self.event_sender {
            let node_info =
                NodeInfo::new(id, node.node_meta().clone(), node.default_inputs().to_vec());
            s.push_node_added(node_info);
        }
        self.nodes.insert(id, node);
    }

    pub fn add_param(&mut self, value: f32) -> NodeInfo {
        self.add_param_with_id(NodeId::new(), value)
    }

    pub fn add_param_with_id(&mut self, id: NodeId, value: f32) -> NodeInfo {
        let param = Param::from(value);
        let param_handle = param.handle();
        let graph_node = GraphNode::new(id, param);
        let node_meta = graph_node.node_meta().clone();
        let default_inputs = graph_node.default_inputs().to_vec();

        self.nodes.insert(id, graph_node);
        self.param_handles.insert(id, param_handle);
        let node_info = NodeInfo::new(id, node_meta, default_inputs);
        if let Some(s) = &self.event_sender {
            s.push_node_added(node_info.clone());
        }
        node_info
    }

    pub fn set_param_value(&mut self, node_id: &NodeId, value: f32) -> Result<(), GraphError> {
        self.node_exists(node_id)?;
        if let Some(handle) = self.param_handles.get_mut(node_id) {
            handle.set(value);
            Ok(())
        } else {
            Err(GraphError::NotAParameter { node_id: *node_id })
        }
    }

    pub fn remove_node(&mut self, node: &NodeId) -> Result<(), GraphError> {
        self.node_exists(node)?;
        if let Some(s) = &self.event_sender {
            let node_info = self.info(node)?;
            s.push_node_removed(node_info);
        }
        self.nodes.remove(node);
        self.connections
            .retain(|conn| &conn.from != node && &conn.to != node);
        Ok(())
    }

    pub fn info(&self, node_id: &NodeId) -> Result<NodeInfo, GraphError> {
        self.node_exists(node_id)?;
        let node = &self.nodes[node_id];
        let node_meta = node.node_meta().clone();
        Ok(NodeInfo::new(
            *node_id,
            node_meta,
            node.default_inputs().to_vec(),
        ))
    }

    pub fn connect(
        &mut self,
        from: &NodeId,
        from_port: usize,
        to: &NodeId,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.port_exists(from, from_port, PortType::Output)?;
        if from == to {
            return Err(GraphError::ImpossibleConnection {
                from_node_id: *from,
                from_port,
                to_node_id: *to,
                to_port,
            });
        }
        self.port_is_free(to, to_port)?;

        self.connections.push(Connection {
            from: *from,
            from_port,
            to: *to,
            to_port,
        });
        if let Some(s) = &self.event_sender {
            let from_port_info = self.port_info(from, from_port, PortType::Output).unwrap();
            let to_port_info = self.port_info(to, to_port, PortType::Input).unwrap();
            s.push_connected(from_port_info, to_port_info);
        }
        Ok(())
    }

    pub fn disconnect(
        &mut self,
        from: &NodeId,
        from_port: usize,
        to: &NodeId,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.port_exists(from, from_port, PortType::Output)?;
        self.port_exists(to, to_port, PortType::Input)?;

        self.connections.retain(|conn| {
            !(&conn.from == from
                && conn.from_port == from_port
                && &conn.to == to
                && conn.to_port == to_port)
        });
        if let Some(s) = &self.event_sender {
            let from_port_info = self.port_info(from, from_port, PortType::Output).unwrap();
            let to_port_info = self.port_info(to, to_port, PortType::Input).unwrap();
            s.push_disconnected(from_port_info, to_port_info);
        }
        Ok(())
    }

    pub fn port_info(
        &self,
        node_id: &NodeId,
        port: usize,
        port_type: PortType,
    ) -> Option<PortInfo> {
        let node = self.nodes.get(node_id)?;
        node.port_info(port_type, port)
    }

    pub fn set_default_input_value(
        &mut self,
        node_id: &NodeId,
        port: usize,
        value: f32,
    ) -> Result<(), GraphError> {
        self.port_exists(node_id, port, PortType::Input)?;
        self.nodes
            .get_mut(node_id)
            .expect("node exists — checked by port_exists")
            .set_default_input_value(port, value);
        Ok(())
    }

    fn node_exists(&self, node_id: &NodeId) -> Result<(), GraphError> {
        self.nodes
            .get(node_id)
            .ok_or(GraphError::NodeNotFound { node_id: *node_id })?;
        Ok(())
    }

    fn port_exists(
        &self,
        node_id: &NodeId,
        port: usize,
        port_type: PortType,
    ) -> Result<(), GraphError> {
        let node = self
            .nodes
            .get(node_id)
            .ok_or(GraphError::NodeNotFound { node_id: *node_id })?;
        node.port_info(port_type, port).map_or(
            Err(GraphError::PortNotFound {
                node_id: *node_id,
                port,
                port_type,
            }),
            |_| Ok(()),
        )
    }

    fn port_is_free(&self, node_id: &NodeId, port: usize) -> Result<(), GraphError> {
        self.port_exists(node_id, port, PortType::Input)?;
        for conn in &self.connections {
            if &conn.to == node_id && conn.to_port == port {
                return Err(GraphError::PortAlreadyConnected {
                    node_id: *node_id,
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
        outputs: &[&PortKey],
        batchsize: usize,
    ) -> impl Iterator<Item = &'a [f32]>;
}

impl BatchTickable for Graph {
    fn tick_batch<'a>(
        &'a mut self,
        output_ports: &[&PortKey],
        batchsize: usize,
    ) -> impl Iterator<Item = &'a [f32]> {
        self.batch_buffer.resize(output_ports, batchsize);
        for sample_index in 0..batchsize {
            self.tick();
            for (port_index, port) in output_ports.iter().enumerate() {
                let val = *self.nodes[port.node_id()].output_value(port.port_index());
                self.batch_buffer.set_value(port_index, sample_index, val);
            }
        }
        (&self.batch_buffer).into_iter()
    }
}

impl PortValueAccess for Graph {
    fn port_value(&self, port: &PortKey) -> Option<&f32> {
        self.nodes
            .get(port.node_id())
            .and_then(|node| node.port_value(port.port_type(), port.port_index()))
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use super::{Graph, PortValueAccess, Tickable};
    use crate::graph::{Node, Param, PortKey, PortType};
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
        let constant_info = graph.add_param(0.5);
        assert_eq!(constant_info.node_type(), type_name::<Param>());
    }

    #[test]
    fn graph_connects_nodes() {
        let mut graph = Graph::new(1);
        let constant_id = *graph.add_param(0.5).node_id();
        let constant_id2 = *graph.add_param(0.5).node_id();
        let adder_id = *graph.add::<Add>().node_id();
        graph
            .connect(&constant_id, 0, &adder_id, 0)
            .expect("Failed to connect nodes");
        graph
            .connect(&constant_id2, 0, &adder_id, 1)
            .expect("Failed to connect nodes");
        graph.tick();
        let output = graph
            .port_value(&PortKey::new(adder_id, 0, PortType::Output))
            .unwrap();
        assert_eq!(*output, 1.0);
        let input1 = graph
            .port_value(&PortKey::new(adder_id, 0, PortType::Input))
            .unwrap();
        assert_eq!(*input1, 0.5);
    }

    #[test]
    fn graph_connect_fails_when_invalid_port() {
        let mut graph = Graph::new(1);
        let constant_id = *graph.add_param(0.5).node_id();
        let adder_id = *graph.add::<Add>().node_id();
        let result = graph.connect(&constant_id, 1, &adder_id, 0);
        assert!(result.is_err());
    }
}
