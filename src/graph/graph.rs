use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::batch::BatchBuffer;
use super::connection::Connection;
use super::error::GraphError;
use super::node::{CreateNode, GraphNode, NodeInfo};
use super::node_id::NodeId;
use super::param::{Param, ParamHandle};
use super::port::{PortIndex, PortInfo, PortKey, PortType};
use crate::event_queue::EventSender;

pub trait PortValueAccess {
    fn port_value(&self, port: &PortKey) -> Option<&f32>;
}

#[wasm_bindgen]
pub struct Graph {
    nodes: Vec<GraphNode>,
    node_index: HashMap<NodeId, usize>,
    param_handles: HashMap<NodeId, ParamHandle>,
    connections: Vec<Connection>,
    sample_rate: u32,
    batch_buffer: BatchBuffer,
    event_sender: Option<EventSender>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            node_index: HashMap::new(),
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
        self.push_node(id, graph_node);
        NodeInfo::new(id, node_meta, default_inputs)
    }

    pub fn insert_node(&mut self, id: NodeId, node: GraphNode) {
        if let Some(s) = &self.event_sender {
            let node_info =
                NodeInfo::new(id, node.node_meta().clone(), node.default_inputs().to_vec());
            s.push_node_added(node_info);
        }
        self.push_node(id, node);
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

        self.push_node(id, graph_node);
        self.param_handles.insert(id, param_handle);
        let node_info = NodeInfo::new(id, node_meta, default_inputs);
        if let Some(s) = &self.event_sender {
            s.push_node_added(node_info.clone());
        }
        node_info
    }

    fn push_node(&mut self, id: NodeId, node: GraphNode) {
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.node_index.insert(id, idx);
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
        let idx = *self.node_index.get(node).expect("checked by node_exists");
        let last_idx = self.nodes.len() - 1;
        self.nodes.swap_remove(idx);
        self.node_index.remove(node);
        if idx != last_idx {
            let moved_id = self.nodes[idx].node_id();
            self.node_index.insert(moved_id, idx);
        }
        let removed = idx;
        let moved = last_idx;
        self.connections.retain_mut(|c| {
            if c.from_idx == removed || c.to_idx == removed {
                return false;
            }
            if c.from_idx == moved {
                c.from_idx = removed;
            }
            if c.to_idx == moved {
                c.to_idx = removed;
            }
            true
        });
        self.param_handles.remove(node);
        Ok(())
    }

    pub fn info(&self, node_id: &NodeId) -> Result<NodeInfo, GraphError> {
        let node = self.node_by_id(node_id)?;
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
        from_port: PortIndex,
        to: &NodeId,
        to_port: PortIndex,
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

        let from_idx = self.node_index[from];
        let to_idx = self.node_index[to];
        self.connections.push(Connection {
            from_idx,
            from_port,
            to_idx,
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
        from_port: PortIndex,
        to: &NodeId,
        to_port: PortIndex,
    ) -> Result<(), GraphError> {
        self.port_exists(from, from_port, PortType::Output)?;
        self.port_exists(to, to_port, PortType::Input)?;

        let from_idx = self.node_index[from];
        let to_idx = self.node_index[to];
        self.connections.retain(|c| {
            !(c.from_idx == from_idx
                && c.from_port == from_port
                && c.to_idx == to_idx
                && c.to_port == to_port)
        });
        if let Some(s) = &self.event_sender {
            let from_port_info = self.port_info(from, from_port, PortType::Output).unwrap();
            let to_port_info = self.port_info(to, to_port, PortType::Input).unwrap();
            s.push_disconnected(from_port_info, to_port_info);
        }
        Ok(())
    }

    pub fn port_info(&self, node_id: &NodeId, port: PortIndex, port_type: PortType) -> Option<PortInfo> {
        let idx = *self.node_index.get(node_id)?;
        self.nodes[idx].port_info(port_type, port)
    }

    pub fn set_default_input_value(
        &mut self,
        node_id: &NodeId,
        port: PortIndex,
        value: f32,
    ) -> Result<(), GraphError> {
        self.port_exists(node_id, port, PortType::Input)?;
        let idx = self.node_index[node_id];
        self.nodes[idx].set_default_input_value(port, value);
        Ok(())
    }

    fn node_by_id(&self, node_id: &NodeId) -> Result<&GraphNode, GraphError> {
        let idx = self
            .node_index
            .get(node_id)
            .ok_or(GraphError::NodeNotFound { node_id: *node_id })?;
        Ok(&self.nodes[*idx])
    }

    fn node_exists(&self, node_id: &NodeId) -> Result<(), GraphError> {
        self.node_index
            .get(node_id)
            .ok_or(GraphError::NodeNotFound { node_id: *node_id })?;
        Ok(())
    }

    fn port_exists(
        &self,
        node_id: &NodeId,
        port: PortIndex,
        port_type: PortType,
    ) -> Result<(), GraphError> {
        let node = self.node_by_id(node_id)?;
        node.port_info(port_type, port).map_or(
            Err(GraphError::PortNotFound {
                node_id: *node_id,
                port,
                port_type,
            }),
            |_| Ok(()),
        )
    }

    fn port_is_free(&self, node_id: &NodeId, port: u8) -> Result<(), GraphError> {
        self.port_exists(node_id, port, PortType::Input)?;
        let to_idx = self.node_index[node_id];
        for c in &self.connections {
            if c.to_idx == to_idx && c.to_port == port {
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
        for node in &mut self.nodes {
            node.reset_input_cache();
        }
        for c in &self.connections {
            let value = *self.nodes[c.from_idx as usize].output_value(c.from_port);
            self.nodes[c.to_idx as usize].set_input_value(c.to_port, value);
        }
        for node in &mut self.nodes {
            node.tick();
        }
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
                let val = self
                    .node_index
                    .get(port.node_id())
                    .map(|&idx| *self.nodes[idx].output_value(port.port_index()))
                    .unwrap_or(0.0);
                self.batch_buffer.set_value(port_index, sample_index, val);
            }
        }
        (&self.batch_buffer).into_iter()
    }
}

impl PortValueAccess for Graph {
    fn port_value(&self, port: &PortKey) -> Option<&f32> {
        let idx = *self.node_index.get(port.node_id())?;
        self.nodes[idx].port_value(port.port_type(), port.port_index())
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

    #[test]
    fn remove_middle_node_preserves_other_connections() {
        // Three nodes a -> c. Add b in the middle, remove b. The a->c edge must still route.
        let mut graph = Graph::new(1);
        let a = *graph.add_param(0.25).node_id();
        let b = *graph.add_param(0.5).node_id();
        let c = *graph.add::<Add>().node_id();
        graph.connect(&a, 0, &c, 0).unwrap();
        // sanity: before remove, b exists at a known idx; removing it triggers swap_remove
        graph.remove_node(&b).expect("remove b");
        graph.tick();
        let out = graph
            .port_value(&PortKey::new(c, 0, PortType::Output))
            .unwrap();
        assert_eq!(*out, 0.25);
    }

    // Run with: cargo test --release --lib bench_tick_batch -- --ignored --nocapture
    #[test]
    #[ignore]
    fn bench_tick_batch() {
        use crate::graph::BatchTickable;
        use std::time::Instant;

        let mut graph = Graph::new(44100);
        let mut adders = Vec::new();
        for _ in 0..50 {
            let p1 = *graph.add_param(0.5).node_id();
            let p2 = *graph.add_param(0.25).node_id();
            let a = *graph.add::<Add>().node_id();
            graph.connect(&p1, 0, &a, 0).unwrap();
            graph.connect(&p2, 0, &a, 1).unwrap();
            adders.push(a);
        }
        let output_port = PortKey::new(adders[0], 0, PortType::Output);
        let ports = [output_port];

        // warm up
        for _ in 0..10 {
            let _: Vec<_> = graph.tick_batch(&ports, 128).collect();
        }

        let iters = 1000;
        let start = Instant::now();
        for _ in 0..iters {
            let _: Vec<_> = graph.tick_batch(&ports, 128).collect();
        }
        let elapsed = start.elapsed();
        let ns_per_block = elapsed.as_nanos() as f64 / iters as f64;
        let budget_ns = 1_000_000_000.0 / 44100.0 * 128.0;
        eprintln!(
            "tick_batch: {:.0} ns/block over {} iters ({:.1}% of {:.0} ns budget)",
            ns_per_block,
            iters,
            ns_per_block / budget_ns * 100.0,
            budget_ns
        );
    }

    #[test]
    fn remove_node_drops_edges_touching_it() {
        let mut graph = Graph::new(1);
        let a = *graph.add_param(0.25).node_id();
        let b = *graph.add::<Add>().node_id();
        let c = *graph.add::<Add>().node_id();
        graph.connect(&a, 0, &b, 0).unwrap();
        graph.connect(&a, 0, &c, 0).unwrap();
        graph.remove_node(&b).expect("remove b");
        // Now port 0 on c should still be free for a fresh connection? No — it's still connected from a.
        // Verify by attempting to connect a->c[0] again and expecting failure.
        let result = graph.connect(&a, 0, &c, 0);
        assert!(
            result.is_err(),
            "c[0] should still be occupied after removing b"
        );
        // And reading the value should still work.
        graph.tick();
        let out = graph
            .port_value(&PortKey::new(c, 0, PortType::Output))
            .unwrap();
        assert_eq!(*out, 0.25);
    }
}
