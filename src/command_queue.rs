use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::{
    graph::{Graph, NodeId, PortIndex, PortKey},
    latest_value::LatestValueWriter,
    node_data::NodeDataWriter,
    nodes::NodeRegistry,
};

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum GraphCommand {
    AddNode {
        id: NodeId,
        typename: String,
    },
    AddParam {
        id: NodeId,
        value: f32,
    },
    RemoveNode(NodeId),
    Connect {
        from: NodeId,
        from_port: PortIndex,
        to: NodeId,
        to_port: PortIndex,
    },
    Disconnect {
        from: NodeId,
        from_port: PortIndex,
        to: NodeId,
        to_port: PortIndex,
    },
    SetParamValue {
        id: NodeId,
        value: f32,
    },
    SetDefaultInputValue {
        id: NodeId,
        port: PortIndex,
        value: f32,
    },
    SubscribeLatestValue {
        #[tsify(type = "string")]
        port: PortKey,
        index: usize,
    },
    UnsubscribeLatestValue {
        index: usize,
    },
    SubscribeNodeData {
        #[tsify(type = "string")]
        port: PortKey,
        index: usize,
    },
    UnsubscribeNodeData {
        index: usize,
    },
}

pub fn apply_command(
    cmd: GraphCommand,
    graph: &mut Graph,
    lv_writer: &mut LatestValueWriter,
    nd_writer: &mut NodeDataWriter,
    node_reg: &NodeRegistry,
    sample_rate: u32,
) {
    match cmd {
        GraphCommand::AddNode { id, typename } => {
            if let Some(node) = node_reg.create(id, &typename, sample_rate) {
                graph.insert_node(id, node);
            }
        }
        GraphCommand::AddParam { id, value } => {
            graph.add_param_with_id(id, value);
        }
        GraphCommand::RemoveNode(id) => {
            let _ = graph.remove_node(&id);
        }
        GraphCommand::Connect {
            from,
            from_port,
            to,
            to_port,
        } => {
            let _ = graph.connect(&from, from_port, &to, to_port);
        }
        GraphCommand::Disconnect {
            from,
            from_port,
            to,
            to_port,
        } => {
            let _ = graph.disconnect(&from, from_port, &to, to_port);
        }
        GraphCommand::SetParamValue { id, value } => {
            let _ = graph.set_param_value(&id, value);
        }
        GraphCommand::SetDefaultInputValue { id, port, value } => {
            let _ = graph.set_default_input_value(&id, port, value);
        }
        GraphCommand::SubscribeLatestValue { port, index } => lv_writer.subscribe(port, index),
        GraphCommand::UnsubscribeLatestValue { index } => lv_writer.unsubscribe(index),
        GraphCommand::SubscribeNodeData { port, index } => nd_writer.subscribe(port, index),
        GraphCommand::UnsubscribeNodeData { index } => nd_writer.unsubscribe(index),
    }
}
