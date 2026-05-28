use wasm_bindgen::prelude::*;

use super::port::{PortInfo, PortType};

#[wasm_bindgen]
pub struct NodeInfo {
    input_names: Vec<&'static str>,
    output_names: Vec<&'static str>,
    node_type: &'static str,
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
        self.node_type.to_string()
    }

    #[wasm_bindgen(getter, js_name = defaultInputValues)]
    pub fn default_input_values(&self) -> Vec<f32> {
        self.default_input_values.clone()
    }
}

pub trait Node<const IN: usize, const OUT: usize> {
    const INPUT_NAMES: [&'static str; IN];
    const OUTPUT_NAMES: [&'static str; OUT];
    fn process(&mut self, input: [f32; IN], output: &mut [f32; OUT]);
    fn new(sample_rate: u32) -> Self;
}

pub struct NodeMeta {
    pub input_names: &'static [&'static str],
    pub output_names: &'static [&'static str],
    pub node_type: &'static str,
}

pub trait DynNode: Send + Sync {
    fn process_sample(&mut self, inputs: &[f32], outputs: &mut [f32]);
    fn meta(&self) -> NodeMeta;
}

struct DynNodeWrapper<const IN: usize, const OUT: usize, T: Node<IN, OUT>>(pub T);

impl<const IN: usize, const OUT: usize, T: Node<IN, OUT> + Send + Sync> DynNode
    for DynNodeWrapper<IN, OUT, T>
{
    fn process_sample(&mut self, input: &[f32], output: &mut [f32]) {
        let mut in_array = [0.0; IN];
        in_array.copy_from_slice(&input[0..IN]);
        let mut out_array = [0.0; OUT];
        self.0.process(in_array, &mut out_array);
        output.copy_from_slice(&out_array[0..OUT]);
    }

    fn meta(&self) -> NodeMeta {
        NodeMeta {
            input_names: &T::INPUT_NAMES,
            output_names: &T::OUTPUT_NAMES,
            node_type: std::any::type_name::<T>(),
        }
    }
}

#[wasm_bindgen]
pub struct GraphNode {
    node: Box<dyn DynNode>,
    input_cache: Box<[f32]>,
    output_cache: Box<[f32]>,
    default_inputs: Box<[f32]>,
}

impl GraphNode {
    pub fn new<const IN: usize, const OUT: usize, T>(node: T) -> GraphNode
    where
        T: Node<IN, OUT> + Sync + Send + 'static,
    {
        let mut node: Box<dyn DynNode> = Box::new(DynNodeWrapper::<IN, OUT, T>(node));
        let default_inputs = Box::new([0.0; IN]);
        let input_cache = default_inputs.clone();
        let mut output_cache = Box::new([0.0; OUT]);
        node.process_sample(&*input_cache, output_cache.as_mut());

        GraphNode {
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
        self.input_cache.len()
    }

    pub fn reset_input_cache(&mut self) {
        self.input_cache.copy_from_slice(&self.default_inputs);
    }

    pub fn set_input_value(&mut self, index: usize, value: f32) {
        if index < self.input_cache.len() {
            self.input_cache[index] = value;
        }
    }

    pub fn node_info(&self) -> NodeInfo {
        let meta = self.node.meta();
        NodeInfo {
            input_names: meta.input_names.to_vec(),
            output_names: meta.output_names.to_vec(),
            node_type: meta.node_type,
            default_input_values: (*self.default_inputs).to_vec(),
        }
    }

    pub fn tick(&mut self) {
        self.node
            .process_sample(&self.input_cache, self.output_cache.as_mut());
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
        let meta = self.node.meta();
        let names = match port_type {
            PortType::Input => meta.input_names,
            PortType::Output => meta.output_names,
        };
        names.get(port).map(|&name| PortInfo {
            port_index: port,
            port_type,
            name,
        })
    }

    pub fn set_default_input_value(&mut self, port: usize, value: f32) {
        if port < self.default_inputs.len() {
            self.default_inputs[port] = value;
        }
    }
}

pub trait CreateNode: Send + 'static {
    fn create(sample_rate: u32) -> GraphNode;
}
