use std::ops::Deref;

pub trait Node<const IN: usize, const OUT: usize> {
    const INPUT_NAMES: [&'static str; IN];
    const OUTPUT_NAMES: [&'static str; OUT];
    fn process(&mut self, input: [f32; IN], sample_num: usize) -> [f32; OUT];
}

pub trait DynNode: Send {
    fn input_names(&self) -> &[&'static str];
    fn output_names(&self) -> &[&'static str];
    fn process(&mut self, input: &[f32], sample_num: usize) -> Vec<f32>;
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
}

struct GraphNode {
    inputs: usize,
    node: Box<dyn DynNode>,
    output_cache: Vec<f32>,
}

impl GraphNode {
    fn new<const IN: usize, const OUT: usize, T>(node: T) -> GraphNode
    where
        T: Node<IN, OUT> + Send + 'static,
    {
        GraphNode {
            inputs: IN,
            node: Box::new(DynNodeWrapper::<IN, OUT, T>(node)),
            output_cache: vec![0.0; OUT],
        }
    }
}

struct Connection {
    from: NodeId,
    from_port: usize,
    to: NodeId,
    to_port: usize,
}

pub struct Graph {
    nodes: Vec<GraphNode>,
    connections: Vec<Connection>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node<const IN: usize, const OUT: usize>(
        &mut self,
        node: impl Node<IN, OUT> + Send + 'static,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        let graph_node = GraphNode::new(node);
        self.nodes.push(graph_node);
        id
    }

    pub fn connect(&mut self, from: NodeId, from_port: usize, to: NodeId, to_port: usize) {
        self.connections.push(Connection {
            from,
            from_port,
            to,
            to_port,
        });
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
        let constant_id = graph.add_node(Param::from(0.5).node());
        let adder_id = graph.add_node(Adder { value: 0.25 });
        assert_eq!(constant_id.0, 0);
        assert_eq!(adder_id.0, 1);
    }

    #[test]
    fn graph_connects_nodes() {
        let mut graph = Graph::new();
        let constant_id = graph.add_node(Param::from(0.5).node());
        let adder_id = graph.add_node(Adder { value: 0.25 });
        graph.connect(constant_id, 0, adder_id, 0);
        graph.tick(0);
        let output = graph.output(adder_id);
        assert_eq!(output, &[0.75]);
    }
}
