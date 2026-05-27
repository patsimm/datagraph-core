use std::str::FromStr;

use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

use crate::graph::{
    BatchTickable, Graph, GraphError, GraphNode, NodeId, NodeInfo, PortKey, PortType,
    PortValueAccess,
};

pub mod event_buffer;
pub mod frequency;
pub mod graph;
pub mod helpers;
pub mod nodes;
pub mod note;
pub mod ramp;
pub mod ring_buffer;
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
        let id = node_id.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: node_id })?;
        self.remove_node(id)
    }

    #[wasm_bindgen(js_name = connect)]
    pub fn _connect(
        &mut self,
        from: String,
        from_port: usize,
        to: String,
        to_port: usize,
    ) -> Result<(), GraphError> {
        let from_id = from.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: from })?;
        let to_id = to.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: to })?;
        self.connect(from_id, from_port, to_id, to_port)
    }

    #[wasm_bindgen(js_name = disconnect)]
    pub fn _disconnect(
        &mut self,
        from: String,
        from_port: usize,
        to: String,
        to_port: usize,
    ) -> Result<(), GraphError> {
        let from_id = from.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: from })?;
        let to_id = to.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: to })?;
        self.disconnect(from_id, from_port, to_id, to_port)
    }

    #[wasm_bindgen(js_name = processBatch)]
    pub fn _process_batch(
        &mut self,
        output_ports: Vec<String>,
        output_buffers: Vec<Float32Array>,
    ) -> Result<(), GraphError> {
        assert!(
            output_ports.len() == output_buffers.len(),
            "output_ports and output_buffers must have the same length"
        );
        let batchsize = output_buffers
            .first()
            .map(|arr| arr.length() as usize)
            .unwrap_or(0);
        if batchsize == 0 {
            return Ok(());
        }

        let port_keys: Vec<PortKey> = output_ports
            .iter()
            .map(|s| PortKey::from_str(s))
            .collect::<Result<_, _>>()?;

        let outputs = self.tick_batch(&port_keys, batchsize);

        for (arr, output) in output_buffers.iter().zip(outputs) {
            arr.copy_from(output);
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = portValue)]
    pub fn _port_value(
        &mut self,
        node_id: String,
        port: usize,
        port_type: PortType,
    ) -> Option<f32> {
        let id = node_id.parse::<NodeId>().ok()?;
        self.port_value(id, port, port_type).copied()
    }

    #[wasm_bindgen(js_name = nodeInfo)]
    pub fn _node_info(&self, node_id: String) -> Result<NodeInfo, GraphError> {
        let id = node_id.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: node_id })?;
        self.info(id)
    }

    #[wasm_bindgen(js_name = setDefaultInputValue)]
    pub fn _set_default_input_value(
        &mut self,
        node_id: String,
        port: usize,
        value: f32,
    ) -> Result<(), GraphError> {
        let id = node_id.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: node_id })?;
        self.set_default_input_value(id, port, value)
    }

    #[wasm_bindgen(js_name = addParam)]
    pub fn _add_param(&mut self, value: f32) -> String {
        self.add_param(value).to_string()
    }

    #[wasm_bindgen(js_name = setParamValue)]
    pub fn _set_param_value(&mut self, node_id: String, value: f32) -> Result<(), GraphError> {
        let id = node_id.parse::<NodeId>().map_err(|_| GraphError::InvalidNodeId { id: node_id })?;
        self.set_param_value(id, value)
    }
}

#[wasm_bindgen]
pub enum DatagraphError {
    NodeNotFound = 0,
    PortNotFound = 1,
    PortAlreadyConnected = 2,
    ImpossibleConnection = 3,
    NotAParameter = 4,
    InvalidPortKey = 5,
    InvalidNodeId = 6,
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
            GraphError::NotAParameter { node_id } => {
                arr.push(&JsValue::from(DatagraphError::NotAParameter));
                arr.push(&JsValue::from(node_id.to_string()));
            }
            GraphError::InvalidPortKey { key } => {
                arr.push(&JsValue::from(DatagraphError::InvalidPortKey));
                arr.push(&JsValue::from_str(&key));
            }
            GraphError::InvalidNodeId { id } => {
                arr.push(&JsValue::from(DatagraphError::InvalidNodeId));
                arr.push(&JsValue::from_str(&id));
            }
        };
        arr.into()
    }
}

#[wasm_bindgen(js_name = createGraph)]
pub fn create_graph(sample_rate: u32) -> Graph {
    Graph::new(sample_rate)
}

#[wasm_bindgen(js_name = nodeTypes)]
pub fn node_types() -> Vec<JsValue> {
    nodes::NodeRegistry::global()
        .node_types()
        .map(JsValue::from_str)
        .collect()
}

#[wasm_bindgen(js_name = createNode)]
pub fn create_node(typename: &str, sample_rate: u32) -> Option<GraphNode> {
    nodes::NodeRegistry::global().create(typename, sample_rate)
}
