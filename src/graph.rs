use nanoid::nanoid;
use std::{collections::HashMap, fmt::Display, hash::Hash, ops::Deref};
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
        let mut node = Box::new(DynNodeWrapper::<IN, OUT, T>(node));
        let output_cache = node.process(&vec![0.0; IN], 0);
        GraphNode {
            inputs: IN,
            node,
            output_cache,
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

    pub fn remove(&mut self, node: NodeId) -> Result<(), GraphError> {
        assert_node_exists(self, node)?;
        self.nodes.remove(&node);
        self.connections
            .retain(|conn| conn.from != node && conn.to != node);
        Ok(())
    }

    pub fn info(&self, node: NodeId) -> Result<NodeInfo, GraphError> {
        self.nodes
            .get(&node)
            .map(|node| node.node.node_info())
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

    pub fn tick(&mut self, sample_num: usize) {
        let keys: Vec<NodeId> = self.nodes.keys().cloned().collect();
        let mut all_inputs: HashMap<NodeId, Vec<f32>> = keys
            .iter()
            .map(|&id| (id, vec![0.0; self.nodes[&id].inputs]))
            .collect();
        for conn in &self.connections {
            all_inputs.get_mut(&conn.to).unwrap()[conn.to_port] =
                self.nodes[&conn.from].output_cache[conn.from_port];
        }
        for node_id in keys {
            let inputs = &all_inputs[&node_id];
            let node = self.nodes.get_mut(&node_id).unwrap();
            node.output_cache = node.node.process(inputs, sample_num);
        }
    }

    pub fn output(&self, node_id: NodeId) -> &[f32] {
        &self.nodes[&node_id].output_cache
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
        port: usize,
        port_type: PortType,
    },
    PortAlreadyConnected {
        node_id: NodeId,
        port: usize,
        port_type: PortType,
    },
    ImpossibleConnection {
        from_node_id: NodeId,
        from_port: usize,
        to_node_id: NodeId,
        to_port: usize,
    },
}

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(match self {
            GraphError::NodeNotFound { node_id } => write!(f, "Node not found: {:?}", *node_id)?,
            GraphError::PortNotFound {
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
                    "Port not found: node {:?} has no {} port {}",
                    node_id, port_type_str, port
                )?
            }
            GraphError::PortAlreadyConnected {
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
                    "Port already connected: node {:?} {} port {} is already connected",
                    node_id, port_type_str, port
                )?
            }
            GraphError::ImpossibleConnection {
                from_node_id,
                from_port,
                to_node_id,
                to_port,
            } => write!(
                f,
                "Impossible connection: cannot connect output port {} of node {:?} to input port {} of node {:?}",
                from_port, from_node_id, to_port, to_node_id
            )?,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId([char; 8]);

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId::from_str(&s)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.iter().collect();
        write!(f, "{}", s)
    }
}

impl NodeId {
    pub fn new() -> Self {
        NodeId(nanoid!(8).chars().collect::<Vec<_>>().try_into().unwrap())
    }

    pub fn from_str(s: &str) -> Self {
        if s.len() != 8 {
            return NodeId::invalid();
        }
        let chars = s.chars().collect::<Vec<_>>();
        chars
            .try_into()
            .map_or_else(|_| NodeId::invalid(), |chars| NodeId(chars))
    }

    pub fn invalid() -> Self {
        NodeId(['\0'; 8])
    }
}

impl Deref for NodeId {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        &self.0
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

pub struct Passthrough;

impl Node<1, 1> for Passthrough {
    const INPUT_NAMES: [&'static str; 1] = ["input"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 1], _: usize) -> [f32; 1] {
        input
    }
}

pub struct Add;

impl Node<2, 1> for Add {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], _: usize) -> [f32; 1] {
        [input[0] + input[1]]
    }
}

pub struct Multiply;

impl Node<2, 1> for Multiply {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], _: usize) -> [f32; 1] {
        [input[0] * input[1]]
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use crate::param::{Param, ParamNode};

    use super::*;

    #[test]
    fn constant_outputs_constant() {
        let param: Param = 0.5.into();
        let output = param.node().process([], 0);
        assert_eq!(output, [0.5]);
    }

    #[test]
    fn graph_adds_nodes() {
        let mut graph = Graph::new();
        let constant_id1 = graph.add(Param::from(0.5).node());
        assert_eq!(
            graph.info(constant_id1).unwrap().node_type,
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
        graph.tick(0);
        let output = graph.output(adder_id);
        assert_eq!(output, &[0.75]);
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
