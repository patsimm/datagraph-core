mod error;
mod node;
mod node_id;
mod port;

pub use crate::nodes::{add::Add, multiply::Multiply, passthrough::Passthrough};
pub use error::GraphError;
pub use node::{DynNode, GraphNode, IntoGraphNode, Node, NodeInfo};
pub use node_id::NodeId;
pub use port::{PortInfo, PortType};

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

struct Connection {
    from: NodeId,
    from_port: usize,
    to: NodeId,
    to_port: usize,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Graph {
    nodes: HashMap<NodeId, GraphNode>,
    connections: Vec<Connection>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let id = NodeId::new();
        self.nodes.insert(id, node);
        id
    }

    pub fn add<const IN: usize, const OUT: usize>(
        &mut self,
        node: impl IntoGraphNode<IN, OUT> + Send + 'static,
    ) -> NodeId {
        let graph_node = node.into_graph_node();
        self.add_node(graph_node)
    }

    pub fn remove_node(&mut self, node: NodeId) -> Result<(), GraphError> {
        assert_node_exists(self, node)?;
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
        assert_port_exists(self, from, from_port, PortType::Output)?;
        if from == to {
            return Err(GraphError::ImpossibleConnection {
                from_node_id: from,
                from_port,
                to_node_id: to,
                to_port,
            });
        }
        assert_port_is_free(self, to, to_port, PortType::Input)?;

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
        assert_port_exists(self, from, from_port, PortType::Output)?;
        assert_port_exists(self, to, to_port, PortType::Input)?;

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

    pub fn tick(&mut self) {
        let keys: Vec<NodeId> = self.nodes.keys().cloned().collect();
        let mut all_inputs: HashMap<NodeId, Vec<f32>> = keys
            .iter()
            .map(|&id| (id, self.nodes[&id].default_inputs().to_vec()))
            .collect();
        for conn in &self.connections {
            all_inputs.get_mut(&conn.to).unwrap()[conn.to_port] =
                *self.nodes[&conn.from].output_value(conn.from_port);
        }
        for node_id in keys {
            let node = self.nodes.get_mut(&node_id).unwrap();
            node.process(&all_inputs[&node_id]);
        }
    }

    pub fn port_value(&self, node_id: NodeId, port: usize, port_type: PortType) -> Option<&f32> {
        self.nodes
            .get(&node_id)
            .and_then(|node| node.port_value(port_type, port))
    }

    pub fn set_default_input_value(
        &mut self,
        node_id: NodeId,
        port: usize,
        value: f32,
    ) -> Result<(), GraphError> {
        assert_port_exists(self, node_id, port, PortType::Input)?;
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.set_default_input_value(port, value);
            Ok(())
        } else {
            Err(GraphError::NodeNotFound { node_id })
        }
    }
}

fn assert_node_exists(graph: &Graph, node_id: NodeId) -> Result<(), GraphError> {
    graph
        .nodes
        .get(&node_id)
        .ok_or(GraphError::NodeNotFound { node_id })?;
    Ok(())
}

fn assert_port_exists(
    graph: &Graph,
    node_id: NodeId,
    port: usize,
    port_type: PortType,
) -> Result<(), GraphError> {
    let node = graph
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

fn assert_port_is_free(
    graph: &Graph,
    node_id: NodeId,
    port: usize,
    port_type: PortType,
) -> Result<(), GraphError> {
    assert_port_exists(graph, node_id, port, port_type)?;

    for conn in &graph.connections {
        if conn.to == node_id && conn.to_port == port && port_type == PortType::Input {
            return Err(GraphError::PortAlreadyConnected {
                node_id,
                port,
                port_type,
            });
        }
        if conn.from == node_id && conn.from_port == port && port_type == PortType::Output {
            return Err(GraphError::PortAlreadyConnected {
                node_id,
                port,
                port_type,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use crate::nodes::param::{Param, ParamNode};

    use super::*;

    #[test]
    fn constant_outputs_constant() {
        let param: Param = 0.5.into();
        let output = param.node().process([]);
        assert_eq!(output, [0.5]);
    }

    #[test]
    fn graph_adds_nodes() {
        let mut graph = Graph::new();
        let constant_id1 = graph.add(Param::from(0.5).node());
        assert_eq!(
            graph.info(constant_id1).unwrap().node_type(),
            type_name::<ParamNode>()
        );
    }

    #[test]
    fn graph_connects_nodes() {
        let mut graph = Graph::new();
        let constant_id = graph.add(Param::from(0.5).node());
        let constant_id2 = graph.add(Param::from(0.25).node());
        let adder_id = graph.add(Add);
        graph
            .connect(constant_id, 0, adder_id, 0)
            .expect("Failed to connect nodes");
        graph
            .connect(constant_id2, 0, adder_id, 1)
            .expect("Failed to connect nodes");
        graph.tick();
        let output = graph.port_value(adder_id, 0, PortType::Output).unwrap();
        assert_eq!(*output, 0.75);
        let input1 = graph.port_value(adder_id, 0, PortType::Input).unwrap();
        assert_eq!(*input1, 0.5);
    }

    #[test]
    fn graph_connect_fails_when_invalid_port() {
        let mut graph = Graph::new();
        let constant_id = graph.add(Param::from(0.5).node());
        let adder_id = graph.add(Add);
        let result = graph.connect(constant_id, 1, adder_id, 0);
        assert!(result.is_err());
    }
}
