use wasm_bindgen::prelude::*;

use crate::{
    command_queue::GraphCommand, graph::NodeInfo, latest_value::LatestValueReader,
    node_data::NodeDataReader, nodes::NodeRegistry,
};

#[wasm_bindgen]
pub struct AudioGraph {
    lv_reader: LatestValueReader,
    nd_reader: NodeDataReader,
    output_node: NodeInfo,
    node_reg: NodeRegistry,
    worklet_node: web_sys::AudioWorkletNode,
}

impl AudioGraph {
    pub fn new(
        lv_reader: LatestValueReader,
        nd_reader: NodeDataReader,
        output_node: NodeInfo,
        node_registry: NodeRegistry,
        worklet_node: web_sys::AudioWorkletNode,
    ) -> Self {
        Self {
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
    #[wasm_bindgen(js_name = sendCommand)]
    pub fn send_command(&self, cmd: GraphCommand) {
        if let Ok(port) = self.worklet_node.port()
            && let Ok(val) = serde_wasm_bindgen::to_value(&cmd)
        {
            let _ = port.post_message(&val);
        }
    }

    #[wasm_bindgen(js_name = latestValueBufferPtr)]
    pub fn _latest_value_buffer_ptr(&self) -> usize {
        self.lv_reader.buffer_ptr()
    }

    #[wasm_bindgen(js_name = latestValueMemory)]
    pub fn _latest_value_memory(&self) -> JsValue {
        wasm_bindgen::memory()
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
