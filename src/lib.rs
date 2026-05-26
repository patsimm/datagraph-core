use std::{collections::HashMap, str::FromStr};

use js_sys::{Float32Array, Map};
use wasm_bindgen::prelude::*;

use crate::graph::{
    BatchTickable, Graph, GraphError, GraphNode, NodeInfo, PortKey, PortType, PortValueAccess,
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
        self.remove_node(node_id.into())
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

    #[wasm_bindgen(js_name = processBatch)]
    pub fn _process_batch(&mut self, output_ports: Map) -> Result<(), GraphError> {
        if output_ports.size() == 0 {
            return Ok(());
        }

        let mut parse_error: Option<GraphError> = None;
        let mut entries: Vec<(PortKey, Float32Array)> = Vec::new();
        let mut batchsize = 0usize;

        output_ports.for_each(&mut |value, key| {
            if parse_error.is_some() {
                return;
            }
            let key_str = key.as_string().unwrap_or_default();
            let arr: Float32Array = value.into();
            if batchsize == 0 {
                batchsize = arr.length() as usize;
            }
            match PortKey::from_str(&key_str) {
                Ok(port_key) => entries.push((port_key, arr)),
                Err(e) => parse_error = Some(e),
            }
        });

        if let Some(e) = parse_error {
            return Err(e);
        }

        if entries.is_empty() {
            return Ok(());
        }

        let mut batch_buffer = std::mem::take(&mut self.batch_buffer);
        let total = entries.len() * batchsize;
        batch_buffer.resize(total, 0.0);

        let mut outputs: HashMap<PortKey, &mut [f32]> = entries
            .iter()
            .zip(batch_buffer.chunks_mut(batchsize))
            .map(|((key, _), chunk)| (*key, chunk))
            .collect();
        self.tick_batch(&mut outputs);

        for (port_key, arr) in entries.iter() {
            arr.copy_from(outputs[port_key]);
        }

        drop(outputs);
        self.batch_buffer = batch_buffer;

        Ok(())
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

    #[wasm_bindgen(js_name = setDefaultInputValue)]
    pub fn _set_default_input_value(
        &mut self,
        node_id: String,
        port: usize,
        value: f32,
    ) -> Result<(), GraphError> {
        self.set_default_input_value(node_id.into(), port, value)
    }

    #[wasm_bindgen(js_name = addParam)]
    pub fn _add_param(&mut self, value: f32) -> String {
        self.add_param(value).to_string()
    }

    #[wasm_bindgen(js_name = setParamValue)]
    pub fn _set_param_value(&mut self, node_id: String, value: f32) -> Result<(), GraphError> {
        self.set_param_value(node_id.into(), value)
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
