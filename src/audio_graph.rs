use wasm_bindgen::prelude::*;

use crate::{
    command_queue::CommandSender,
    graph::{GraphError, GraphNode, NodeId, NodeInfo, Param, PortKey, PortType},
    latest_value::LatestValueReader,
    node_data::NodeDataReader,
    nodes::NodeRegistry,
};

#[wasm_bindgen]
pub struct AudioGraph {
    sample_rate: u32,
    sender: CommandSender,
    lv_reader: LatestValueReader,
    nd_reader: NodeDataReader,
    output_node: NodeInfo,
    node_reg: NodeRegistry,
    worklet_node: web_sys::AudioWorkletNode,
}

impl AudioGraph {
    pub fn new(
        sample_rate: u32,
        sender: CommandSender,
        lv_reader: LatestValueReader,
        nd_reader: NodeDataReader,
        output_node: NodeInfo,
        node_registry: NodeRegistry,
        worklet_node: web_sys::AudioWorkletNode,
    ) -> Self {
        Self {
            sample_rate,
            sender,
            lv_reader,
            nd_reader,
            output_node,
            node_reg: node_registry,
            worklet_node,
        }
    }
}

#[wasm_bindgen]
impl AudioGraph {
    #[wasm_bindgen(js_name = add)]
    pub fn _add(&self, typename: &str) -> Result<NodeInfo, String> {
        let id = NodeId::new();
        let node = self
            .node_reg
            .create(id, typename, self.sample_rate)
            .ok_or(format!(
                "failed to create node of type {typename}. available types: {:?}",
                self.node_reg.node_types().collect::<Vec<_>>()
            ))?;
        let meta = node.node_meta();
        let defaults = node.default_inputs().to_vec();
        let info = NodeInfo::new(id, meta, defaults);
        self.sender.add_node_with_id(id, node);
        Ok(info)
    }

    #[wasm_bindgen(js_name = addParam)]
    pub fn _add_param(&self, value: f32) -> NodeInfo {
        let id = NodeId::new();
        let temp_node = GraphNode::new(id, Param::from(0.0_f32));
        let meta = temp_node.node_meta();
        let defaults = temp_node.default_inputs().to_vec();
        let info = NodeInfo::new(id, meta, defaults);
        self.sender.add_param_with_id(id, value);
        info
    }

    #[wasm_bindgen(js_name = remove)]
    pub fn _remove(&self, node_id: String) -> Result<(), GraphError> {
        let id = node_id
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId {
                id: node_id.clone(),
            })?;
        self.sender.remove_node(id);
        Ok(())
    }

    #[wasm_bindgen(js_name = connect)]
    pub fn _connect(
        &self,
        from: String,
        from_port: usize,
        to: String,
        to_port: usize,
    ) -> Result<(), GraphError> {
        let from_id = from
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId { id: from.clone() })?;
        let to_id = to
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId { id: to.clone() })?;
        self.sender.connect(from_id, from_port, to_id, to_port);
        Ok(())
    }

    #[wasm_bindgen(js_name = disconnect)]
    pub fn _disconnect(
        &self,
        from: String,
        from_port: usize,
        to: String,
        to_port: usize,
    ) -> Result<(), GraphError> {
        let from_id = from
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId { id: from.clone() })?;
        let to_id = to
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId { id: to.clone() })?;
        self.sender.disconnect(from_id, from_port, to_id, to_port);
        Ok(())
    }

    #[wasm_bindgen(js_name = setParamValue)]
    pub fn _set_param_value(&self, node_id: String, value: f32) -> Result<(), GraphError> {
        let id = node_id
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId {
                id: node_id.clone(),
            })?;
        self.sender.set_param_value(id, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = setDefaultInputValue)]
    pub fn _set_default_input_value(
        &self,
        node_id: String,
        port: usize,
        value: f32,
    ) -> Result<(), GraphError> {
        let id = node_id
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId {
                id: node_id.clone(),
            })?;
        self.sender.set_default_input_value(id, port, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = subscribeLatestValue)]
    pub fn _subscribe_latest_value(
        &self,
        node_id: String,
        port: usize,
        port_type: PortType,
        index: usize,
    ) -> Result<(), GraphError> {
        let id = node_id
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId {
                id: node_id.clone(),
            })?;
        let port_key = PortKey::new(id, port, port_type);
        self.sender.subscribe_latest_value(port_key, index);
        Ok(())
    }

    #[wasm_bindgen(js_name = unsubscribeLatestValue)]
    pub fn _unsubscribe_latest_value(&self, index: usize) {
        self.sender.unsubscribe_latest_value(index);
    }

    #[wasm_bindgen(js_name = latestValueBufferPtr)]
    pub fn _latest_value_buffer_ptr(&self) -> usize {
        self.lv_reader.buffer_ptr()
    }

    #[wasm_bindgen(js_name = latestValueMemory)]
    pub fn _latest_value_memory(&self) -> JsValue {
        wasm_bindgen::memory()
    }

    #[wasm_bindgen(js_name = subscribeNodeData)]
    pub fn _subscribe_node_data(
        &self,
        node_id: String,
        port: usize,
        port_type: PortType,
        index: usize,
    ) -> Result<(), GraphError> {
        let id = node_id
            .parse::<NodeId>()
            .map_err(|_| GraphError::InvalidNodeId {
                id: node_id.clone(),
            })?;
        let port_key = PortKey::new(id, port, port_type);
        self.sender.subscribe_node_data(port_key, index);
        Ok(())
    }

    #[wasm_bindgen(js_name = unsubscribeNodeData)]
    pub fn _unsubscribe_node_data(&self, index: usize) {
        self.sender.unsubscribe_node_data(index);
    }

    #[wasm_bindgen(js_name = nodeDataBufferPtr)]
    pub fn _node_data_buffer_ptr(&self) -> usize {
        self.nd_reader.buffer_ptr()
    }

    #[wasm_bindgen(js_name = nodeDataBufferStride)]
    pub fn _node_data_buffer_stride(&self) -> usize {
        self.nd_reader.buffer_stride()
    }

    #[wasm_bindgen(js_name = nodeDataMemory)]
    pub fn _node_data_memory(&self) -> JsValue {
        wasm_bindgen::memory()
    }

    #[wasm_bindgen(js_name = outputNode)]
    pub fn _output_node(&self) -> NodeInfo {
        self.output_node.clone()
    }

    #[wasm_bindgen(js_name = nodeTypes)]
    pub fn _node_types(&self) -> Vec<JsValue> {
        self.node_reg.node_types().map(JsValue::from_str).collect()
    }

    #[wasm_bindgen(getter, js_name = workletNode)]
    pub fn worklet_node(&self) -> web_sys::AudioWorkletNode {
        self.worklet_node.clone()
    }
}
