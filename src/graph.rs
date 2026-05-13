use std::{fmt::Display, ops::Deref};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NodeInfo {
    input_names: Vec<&'static str>,
    output_names: Vec<&'static str>,
    node_type: String,
}

#[wasm_bindgen]
impl NodeInfo {
    #[wasm_bindgen(getter, js_name = inputNames)]
    pub fn input_names(&self) -> Vec<JsValue> {
        self.input_names
            .iter()
            .map(|s| JsValue::from_str(s))
            .collect()
    }

    #[wasm_bindgen(getter, js_name = outputNames)]
    pub fn output_names(&self) -> Vec<JsValue> {
        self.output_names
            .iter()
            .map(|s| JsValue::from_str(s))
            .collect()
    }

    #[wasm_bindgen(getter, js_name = nodeType)]
    pub fn node_type(&self) -> String {
        self.node_type.clone()
    }
}

pub trait Node<const IN: usize, const OUT: usize> {
    const INPUT_NAMES: [&'static str; IN];
    const OUTPUT_NAMES: [&'static str; OUT];
    fn process(&mut self, input: [f32; IN], sample_num: usize) -> [f32; OUT];
}

pub trait DynNode: Send {
    fn input_names(&self) -> &[&'static str];
    fn output_names(&self) -> &[&'static str];
    fn process(&mut self, input: &[f32], sample_num: usize) -> Vec<f32>;
    fn node_type(&self) -> String;
    fn node_info(&self) -> NodeInfo {
        NodeInfo {
            input_names: self.input_names().to_vec(),
            output_names: self.output_names().to_vec(),
            node_type: self.node_type(),
        }
    }
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
    fn node_type(&self) -> String {
        std::any::type_name::<T>().to_string()
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

    pub fn info(&self, node_id: NodeId) -> Option<NodeInfo> {
        self.nodes.get(*node_id).map(|node| node.node.node_info())
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
                from_node_type: self.nodes[*from].node.node_type(),
                from_port,
                to_node_id: to,
                to_node_type: self.nodes[*to].node.node_type(),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug)]
pub enum GraphError {
    NodeNotFound {
        node_id: NodeId,
    },
    PortNotFound {
        node_id: NodeId,
        node_type: String,
        port: usize,
        port_type: PortType,
    },
    PortAlreadyConnected {
        node_id: NodeId,
        node_type: String,
        port: usize,
        port_type: PortType,
    },
    ImpossibleConnection {
        from_node_id: NodeId,
        from_node_type: String,
        from_port: usize,
        to_node_id: NodeId,
        to_node_type: String,
        to_port: usize,
    },
}

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(match self {
            GraphError::NodeNotFound { node_id } => write!(f, "Node not found: {:?}", *node_id)?,
            GraphError::PortNotFound {
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
            GraphError::PortAlreadyConnected {
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
                    "Port already connected: {:?} node {:?}, {} port {}",
                    node_type, *node_id, port_type_str, port
                )?
            }
            GraphError::ImpossibleConnection {
                from_node_id,
                from_node_type,
                from_port,
                to_node_id,
                to_node_type,
                to_port,
            } => write!(
                f,
                "Impossible connection: {:?} node {:?} port {} to {:?} node {:?} port {}",
                from_node_type, *from_node_id, from_port, to_node_type, *to_node_id, to_port
            )?,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub usize);

impl Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn assert_port_exists(
    graph: &Graph,
    node_id: NodeId,
    port: usize,
    port_type: PortType,
) -> Result<(), GraphError> {
    let node = graph
        .nodes
        .get(*node_id)
        .ok_or(GraphError::NodeNotFound { node_id })?;

    node.port_info(port_type, port).map_or(
        Err(GraphError::PortNotFound {
            node_id,
            node_type: node.node.node_type(),
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

    let graph_node = graph
        .nodes
        .get(*node_id)
        .ok_or(GraphError::NodeNotFound { node_id })?;

    for conn in &graph.connections {
        if conn.to == node_id && conn.to_port == port && port_type == PortType::Input {
            return Err(GraphError::PortAlreadyConnected {
                node_id,
                node_type: graph_node.node.node_type(),
                port,
                port_type,
            });
        }
        if conn.from == node_id && conn.from_port == port && port_type == PortType::Output {
            return Err(GraphError::PortAlreadyConnected {
                node_id,
                node_type: graph_node.node.node_type(),
                port,
                port_type,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::param::Param;

    use super::*;

    struct Adder {
        value: f32,
    }

    impl Node<1, 1> for Adder {
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
