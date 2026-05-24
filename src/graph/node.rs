use wasm_bindgen::prelude::*;

use super::port::{PortInfo, PortType};

#[wasm_bindgen]
pub struct NodeInfo {
    input_names: Vec<&'static str>,
    output_names: Vec<&'static str>,
    node_type: String,
    default_input_values: Vec<f32>,
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

    #[wasm_bindgen(getter, js_name = defaultInputValues)]
    pub fn default_input_values(&self) -> Vec<f32> {
        self.default_input_values.clone()
    }
}

pub trait Node<const IN: usize, const OUT: usize> {
    const INPUT_NAMES: [&'static str; IN];
    const OUTPUT_NAMES: [&'static str; OUT];
    fn process(&mut self, input: [f32; IN]) -> [f32; OUT];
}

pub trait DynNode: Send {
    fn input_names(&self) -> &[&'static str];
    fn output_names(&self) -> &[&'static str];
    fn process(&mut self, input: &[f32]) -> Vec<f32>;
    fn node_type(&self) -> String;
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
    fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut in_array = [0.0; IN];
        in_array.copy_from_slice(&input[0..IN]);
        let out_array = self.0.process(in_array);
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
    input_cache: Vec<f32>,
    output_cache: Vec<f32>,
    default_inputs: Vec<f32>,
}

impl GraphNode {
    pub fn from<const IN: usize, const OUT: usize, T>(node: T) -> GraphNode
    where
        T: Node<IN, OUT> + Send + 'static,
    {
        let mut node = Box::new(DynNodeWrapper::<IN, OUT, T>(node));
        let default_inputs = vec![0.0; IN];
        let input_cache = default_inputs.clone();
        let output_cache = node.process(&input_cache);

        GraphNode {
            inputs: IN,
            node,
            output_cache,
            input_cache,
            default_inputs,
        }
    }

    pub fn default_inputs(&self) -> &[f32] {
        &self.default_inputs
    }

    pub fn input_count(&self) -> usize {
        self.inputs
    }

    pub fn node_info(&self) -> NodeInfo {
        NodeInfo {
            input_names: self.node.input_names().to_vec(),
            output_names: self.node.output_names().to_vec(),
            node_type: self.node.node_type(),
            default_input_values: self.default_inputs.clone(),
        }
    }

    pub fn process(&mut self, input: &[f32]) {
        self.input_cache.copy_from_slice(input);
        self.output_cache = self.node.process(input);
    }

    pub fn output_value(&self, port: usize) -> &f32 {
        &self.output_cache[port]
    }

    pub fn input_value(&self, port: usize) -> &f32 {
        &self.input_cache[port]
    }

    pub fn port_value(&self, port_type: PortType, port: usize) -> Option<&f32> {
        match port_type {
            PortType::Input => {
                if port < self.input_cache.len() {
                    Some(&self.input_cache[port])
                } else {
                    None
                }
            }
            PortType::Output => {
                if port < self.output_cache.len() {
                    Some(&self.output_cache[port])
                } else {
                    None
                }
            }
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

    pub fn set_default_input_value(&mut self, port: usize, value: f32) {
        if port < self.default_inputs.len() {
            self.default_inputs[port] = value;
        }
    }
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
