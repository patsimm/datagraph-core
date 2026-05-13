use std::time::Duration;

use wasm_bindgen::prelude::*;

use crate::{
    graph::{Graph, GraphError, GraphNode, NodeId, NodeInfo},
    oscillator::Oscillator,
    param::Param,
};

pub mod delay;
pub mod event_buffer;
pub mod filter;
pub mod frequency;
pub mod gain;
pub mod graph;
pub mod helpers;
pub mod note;
pub mod oscillator;
pub mod param;
pub mod ring_buffer;
pub mod wav;

#[wasm_bindgen]
impl Graph {
    #[wasm_bindgen(js_name = add)]
    pub fn _add(&mut self, node: GraphNode) -> usize {
        *self.add_node(node)
    }

    #[wasm_bindgen(js_name = addParam)]
    pub fn _add_param(&mut self, param: &Param) -> usize {
        *self.add_node(GraphNode::from(param.node()))
    }

    #[wasm_bindgen(js_name = connect)]
    pub fn _connect(
        &mut self,
        from: usize,
        from_port: usize,
        to: usize,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.connect(NodeId(from), from_port, NodeId(to), to_port)
    }

    #[wasm_bindgen(js_name = disconnect)]
    pub fn _disconnect(
        &mut self,
        from: usize,
        from_port: usize,
        to: usize,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.disconnect(NodeId(from), from_port, NodeId(to), to_port)
    }

    #[wasm_bindgen(js_name = tick)]
    pub fn _tick(&mut self, sample_num: usize) {
        self.tick(sample_num);
    }

    #[wasm_bindgen(js_name = output)]
    pub fn _output(&mut self, node_id: usize) -> Vec<f32> {
        self.output(NodeId(node_id)).to_vec()
    }

    #[wasm_bindgen(js_name = nodeInfo)]
    pub fn _node_info(&self, node_id: usize) -> Option<NodeInfo> {
        self.info(NodeId(node_id))
    }
}

#[wasm_bindgen]
pub enum DatagraphError {
    NodeNotFound = 0,
    PortNotFound = 1,
    PortAlreadyConnected = 2,
    ImpossibleConnection = 3,
}

impl From<GraphError> for JsValue {
    fn from(err: GraphError) -> Self {
        let arr = js_sys::Array::new();
        match err {
            GraphError::NodeNotFound { node_id } => {
                arr.push(&JsValue::from(DatagraphError::NodeNotFound));
                arr.push(&JsValue::from(*node_id));
            }
            GraphError::PortNotFound {
                node_id,
                node_type,
                port,
                port_type,
            } => {
                arr.push(&JsValue::from(DatagraphError::PortNotFound));
                arr.push(&JsValue::from(*node_id));
                arr.push(&JsValue::from(node_type));
                arr.push(&JsValue::from(port));
                arr.push(&JsValue::from(port_type));
            }
            GraphError::PortAlreadyConnected {
                node_id,
                node_type,
                port,
                port_type,
            } => {
                arr.push(&JsValue::from(DatagraphError::PortAlreadyConnected));
                arr.push(&JsValue::from(*node_id));
                arr.push(&JsValue::from(node_type));
                arr.push(&JsValue::from(port));
                arr.push(&JsValue::from(port_type));
            }
            GraphError::ImpossibleConnection {
                from_node_id,
                from_node_type,
                from_port,
                to_node_id,
                to_node_type,
                to_port,
            } => {
                arr.push(&JsValue::from(DatagraphError::ImpossibleConnection));
                arr.push(&JsValue::from(*from_node_id));
                arr.push(&JsValue::from(from_node_type));
                arr.push(&JsValue::from(from_port));
                arr.push(&JsValue::from(*to_node_id));
                arr.push(&JsValue::from(to_node_type));
                arr.push(&JsValue::from(to_port));
            }
        };
        arr.into()
    }
}

#[wasm_bindgen(js_name = createGraph)]
pub fn create_graph() -> Graph {
    Graph::new()
}

#[wasm_bindgen(js_name = createOscillator)]
pub fn create_oscillator(sample_rate: u32) -> GraphNode {
    GraphNode::from(Oscillator::new(sample_rate))
}

#[wasm_bindgen(js_name = createParam)]
pub fn create_param(value: f32) -> Param {
    Param::new(value)
}

#[wasm_bindgen]
impl Param {
    #[wasm_bindgen(js_name = set)]
    pub fn _set(&mut self, value: f32) {
        self.set(value);
    }
}

#[wasm_bindgen(js_name = createGain)]
pub fn create_gain() -> GraphNode {
    GraphNode::from(gain::Gain)
}

#[wasm_bindgen(js_name = createADSR)]
pub fn create_adsr(
    sample_rate: u32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
) -> GraphNode {
    GraphNode::from(gain::ADSR::new(
        sample_rate,
        std::time::Duration::from_secs_f32(attack),
        std::time::Duration::from_secs_f32(decay),
        sustain,
        std::time::Duration::from_secs_f32(release),
    ))
}

#[wasm_bindgen(js_name = createOnePoleLowPass)]
pub fn create_one_pole_low_pass(smoothing_ms: u64, sample_rate: u32) -> GraphNode {
    GraphNode::from(filter::OnePoleLowPass::from_smoothing_time(
        Duration::from_millis(smoothing_ms),
        sample_rate,
    ))
}

#[wasm_bindgen(js_name = createDelay)]
pub fn create_delay() -> GraphNode {
    GraphNode::from(delay::Delay::new())
}
