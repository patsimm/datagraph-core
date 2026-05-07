use std::{fmt::Display, ops::Deref};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Unknown = 0,
    Oscillator = 1,
    Param = 2,
    Gain = 3,
    ADSR = 4,
    Delay = 5,
}

pub trait Node<const IN: usize, const OUT: usize> {
    const NODE_TYPE: NodeType;
    const INPUT_NAMES: [&'static str; IN];
    const OUTPUT_NAMES: [&'static str; OUT];
    fn process(&mut self, input: [f32; IN], sample_num: usize) -> [f32; OUT];
}

pub trait DynNode: Send {
    fn input_names(&self) -> &[&'static str];
    fn output_names(&self) -> &[&'static str];
    fn process(&mut self, input: &[f32], sample_num: usize) -> Vec<f32>;
    fn node_type(&self) -> NodeType;
}

struct DynNodeWrapper<const IN: usize, const OUT: usize, T: Node<IN, OUT>>(pub T);

impl<const IN: usize, const OUT: usize, T: Node<IN, OUT> + Send> DynNode
    for DynNodeWrapper<IN, OUT, T>
{
    fn input_names(&self) -> &[&'static str] {
        &T::INPUT_NAMES
    }
    fn output_names(&self) -> &[&'static str] {
        &T::OUTPUT_NAMES
    }
    fn process(&mut self, input: &[f32], sample_num: usize) -> Vec<f32> {
        let mut in_array = [0.0; IN];
        in_array.copy_from_slice(&input[0..IN]);
        let out_array = self.0.process(in_array, sample_num);
        out_array.to_vec()
    }
    fn node_type(&self) -> NodeType {
        T::NODE_TYPE
    }
}

#[wasm_bindgen]
pub struct GraphNode {
    inputs: usize,
    node: Box<dyn DynNode>,
    output_cache: Vec<f32>,
}

impl GraphNode {
    pub fn from<const IN: usize, const OUT: usize, T>(node: T) -> GraphNode
    where
        T: Node<IN, OUT> + Send + 'static,
    {
        GraphNode {
            inputs: IN,
            node: Box::new(DynNodeWrapper::<IN, OUT, T>(node)),
            output_cache: vec![0.0; OUT],
        }
    }

    pub fn port_info(&self, port_type: PortType, port: usize) -> Option<PortInfo> {
        match port_type {
            PortType::Input => {
                if port < self.node.input_names().len() {
                    Some(PortInfo {
                        port_index: port,
                        port_type: PortType::Input,
                        name: self.node.input_names().get(port).copied().unwrap_or(""),
                    })
                } else {
                    None
                }
            }
            PortType::Output => {
                if port < self.node.output_names().len() {
                    Some(PortInfo {
                        port_index: port,
                        port_type: PortType::Output,
                        name: self.node.output_names().get(port).copied().unwrap_or(""),
                    })
                } else {
                    None
                }
            }
        }
    }
}

pub struct PortInfo {
    pub port_index: usize,
    pub port_type: PortType,
    pub name: &'static str,
}

pub trait IntoGraphNode<const IN: usize, const OUT: usize> {
    fn into_graph_node(self) -> GraphNode;
}

impl<const IN: usize, const OUT: usize, T: Node<IN, OUT> + Send + 'static> IntoGraphNode<IN, OUT>
    for T
{
    fn into_graph_node(self) -> GraphNode {
        GraphNode::from(self)
    }
}

struct Connection {
    from: NodeId,
    from_port: usize,
    to: NodeId,
    to_port: usize,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Graph {
    nodes: Vec<GraphNode>,
    connections: Vec<Connection>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn add<const IN: usize, const OUT: usize>(
        &mut self,
        node: impl IntoGraphNode<IN, OUT> + Send + 'static,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        let graph_node = node.into_graph_node();
        self.add_node(graph_node);
        id
    }

    pub fn connect(
        &mut self,
        from: NodeId,
        from_port: usize,
        to: NodeId,
        to_port: usize,
    ) -> Result<(), GraphConnectionError> {
        let from_node = self
            .nodes
            .get(*from)
            .ok_or(GraphConnectionError::NodeNotFound { node_id: from })?;
        let to_node = self
            .nodes
            .get(*to)
            .ok_or(GraphConnectionError::NodeNotFound { node_id: to })?;

        if from_node.port_info(PortType::Output, from_port).is_none() {
            return Err(GraphConnectionError::PortNotFound {
                node_type: from_node.node.node_type(),
                node_id: from,
                port: from_port,
                port_type: PortType::Output,
            });
        }

        if to_node.port_info(PortType::Input, to_port).is_none() {
            return Err(GraphConnectionError::PortNotFound {
                node_type: to_node.node.node_type(),
                node_id: to,
                port: to_port,
                port_type: PortType::Input,
            });
        }

        self.connections.push(Connection {
            from,
            from_port,
            to,
            to_port,
        });

        Ok(())
    }

    pub fn port_info(&self, node_id: NodeId, port: usize, port_type: PortType) -> Option<PortInfo> {
        let node = self.nodes.get(*node_id)?;
        node.port_info(port_type, port)
    }

    pub fn tick(&mut self, sample_num: usize) {
        for node_id in 0..self.nodes.len() {
            let mut inputs = vec![0.0; self.nodes[node_id].inputs];
            for conn in &mut self.connections {
                if conn.to != NodeId(node_id) {
                    continue;
                }
                inputs[conn.to_port] = self.nodes[*conn.from].output_cache[conn.from_port];
            }
            let node = &mut self.nodes[node_id];
            node.output_cache = node.node.process(&inputs, sample_num);
        }
    }

    pub fn output(&self, node_id: NodeId) -> &[f32] {
        &self.nodes[*node_id].output_cache
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug)]
pub enum GraphConnectionError {
    NodeNotFound {
        node_id: NodeId,
    },
    PortNotFound {
        node_id: NodeId,
        node_type: NodeType,
        port: usize,
        port_type: PortType,
    },
}

impl Display for GraphConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(match self {
            GraphConnectionError::NodeNotFound { node_id } => {
                write!(f, "Node not found: {:?}", *node_id)?
            }
            GraphConnectionError::PortNotFound {
                node_type,
                node_id,
                port,
                port_type,
            } => {
                let port_type_str = match port_type {
                    PortType::Input => "input",
                    PortType::Output => "output",
                };
                write!(
                    f,
                    "Port not found: {:?} node {:?}, {} port {}",
                    node_type, *node_id, port_type_str, port
                )?
            }
        })
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(usize);

impl Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::param::Param;

    use super::*;

    struct Adder {
        value: f32,
    }

    impl Node<1, 1> for Adder {
        const NODE_TYPE: NodeType = NodeType::Unknown;
        const INPUT_NAMES: [&'static str; 1] = ["input"];
        const OUTPUT_NAMES: [&'static str; 1] = ["output"];
        fn process(&mut self, input: [f32; 1], _: usize) -> [f32; 1] {
            [input[0] + self.value]
        }
    }

    #[test]
    fn constant_outputs_constant() {
        let param: Param = 0.5.into();
        let output = param.node().process([], 0);
        assert_eq!(output, [0.5]);
    }

    #[test]
    fn adder_adds_value() {
        let param: Param = 0.5.into();
        let mut adder = Adder { value: 0.25 };
        let output = adder.process(param.node().process([], 0), 0);
        assert_eq!(output, [0.75]);
    }

    #[test]
    fn graph_adds_nodes() {
        let mut graph = Graph::new();
        let constant_id = graph.add(Param::from(0.5).node());
        let adder_id = graph.add(Adder { value: 0.25 });
        assert_eq!(constant_id.0, 0);
        assert_eq!(adder_id.0, 1);
    }

    #[test]
    fn graph_connects_nodes() {
        let mut graph = Graph::new();
        let constant_id = graph.add(Param::from(0.5).node());
        let adder_id = graph.add(Adder { value: 0.25 });
        graph
            .connect(constant_id, 0, adder_id, 0)
            .expect("Failed to connect nodes");
        graph.tick(0);
        let output = graph.output(adder_id);
        assert_eq!(output, &[0.75]);
    }

    #[test]
    fn graph_connect_fails_when_invalid_port() {
        let mut graph = Graph::new();
        let constant_id = graph.add(Param::from(0.5).node());
        let adder_id = graph.add(Adder { value: 0.25 });
        let result = graph.connect(constant_id, 1, adder_id, 0);
        assert!(result.is_err());
    }
}
