use std::time::Duration;

use wasm_bindgen::prelude::*;

use crate::{
    graph::{Graph, GraphError, GraphNode, NodeInfo, PortType},
    param::Param,
};

pub mod adsr;
pub mod delay;
pub mod event_buffer;
pub mod filter;
pub mod frequency;
pub mod graph;
pub mod helpers;
pub mod note;
pub mod oscillator;
pub mod param;
pub mod ramp;
pub mod ring_buffer;
pub mod sequencer;
pub mod state_machine;
pub mod wav;

#[wasm_bindgen]
impl Graph {
    #[wasm_bindgen(js_name = add)]
    pub fn _add(&mut self, node: GraphNode) -> String {
        self.add_node(node).to_string()
    }

    #[wasm_bindgen(js_name = remove)]
    pub fn _remove(&mut self, node_id: String) -> Result<(), GraphError> {
        self.remove_node(node_id.into())
    }

    #[wasm_bindgen(js_name = addParam)]
    pub fn _add_param(&mut self, param: &Param) -> String {
        self.add_node(GraphNode::from(param.node())).to_string()
    }

    #[wasm_bindgen(js_name = connect)]
    pub fn _connect(
        &mut self,
        from: String,
        from_port: usize,
        to: String,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.connect(from.into(), from_port, to.into(), to_port)
    }

    #[wasm_bindgen(js_name = disconnect)]
    pub fn _disconnect(
        &mut self,
        from: String,
        from_port: usize,
        to: String,
        to_port: usize,
    ) -> Result<(), GraphError> {
        self.disconnect(from.into(), from_port, to.into(), to_port)
    }

    #[wasm_bindgen(js_name = tick)]
    pub fn _tick(&mut self) {
        self.tick();
    }

    #[wasm_bindgen(js_name = portValue)]
    pub fn _port_value(
        &mut self,
        node_id: String,
        port: usize,
        port_type: PortType,
    ) -> Option<f32> {
        self.port_value(node_id.into(), port, port_type).copied()
    }

    #[wasm_bindgen(js_name = nodeInfo)]
    pub fn _node_info(&self, node_id: String) -> Result<NodeInfo, GraphError> {
        self.info(node_id.into())
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
                arr.push(&JsValue::from(node_id.to_string()));
            }
            GraphError::PortNotFound {
                node_id,
                port,
                port_type,
            } => {
                arr.push(&JsValue::from(DatagraphError::PortNotFound));
                arr.push(&JsValue::from(node_id.to_string()));
                arr.push(&JsValue::from(port));
                arr.push(&JsValue::from(port_type));
            }
            GraphError::PortAlreadyConnected {
                node_id,
                port,
                port_type,
            } => {
                arr.push(&JsValue::from(DatagraphError::PortAlreadyConnected));
                arr.push(&JsValue::from(node_id.to_string()));
                arr.push(&JsValue::from(port));
                arr.push(&JsValue::from(port_type));
            }
            GraphError::ImpossibleConnection {
                from_node_id,
                from_port,
                to_node_id,
                to_port,
            } => {
                arr.push(&JsValue::from(DatagraphError::ImpossibleConnection));
                arr.push(&JsValue::from(from_node_id.to_string()));
                arr.push(&JsValue::from(from_port));
                arr.push(&JsValue::from(to_node_id.to_string()));
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

#[wasm_bindgen(js_name = createSin)]
pub fn create_sin(sample_rate: u32) -> GraphNode {
    GraphNode::from(oscillator::Sin::new(sample_rate))
}

#[wasm_bindgen(js_name = createSaw)]
pub fn create_saw(sample_rate: u32) -> GraphNode {
    GraphNode::from(oscillator::Saw::new(sample_rate))
}

#[wasm_bindgen(js_name = createSequencer)]
pub fn create_sequencer() -> GraphNode {
    GraphNode::from(sequencer::Sequencer::default())
}

#[wasm_bindgen(js_name = createSquare)]
pub fn create_square(sample_rate: u32) -> GraphNode {
    GraphNode::from(oscillator::Square::new(sample_rate))
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

#[wasm_bindgen(js_name = createMultiply)]
pub fn create_multiply() -> GraphNode {
    GraphNode::from(graph::Multiply)
}

#[wasm_bindgen(js_name = createADSR)]
pub fn create_adsr(
    sample_rate: u32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
) -> GraphNode {
    GraphNode::from(adsr::ADSR::new(
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

#[wasm_bindgen(js_name = createPassthrough)]
pub fn create_passthrough() -> GraphNode {
    GraphNode::from(graph::Passthrough)
}

#[wasm_bindgen(js_name = createAdd)]
pub fn create_add() -> GraphNode {
    GraphNode::from(graph::Add)
}
