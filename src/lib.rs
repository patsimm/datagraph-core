use wasm_bindgen::prelude::*;
use web_sys::AudioContext;

use crate::{
    audio_graph::AudioGraph,
    event_queue::event_channel,
    graph::{Graph, PortType},
    latest_value::latest_value_channel,
    node_data::node_data_channel,
    nodes::{NodeRegistry, Passthrough},
    wasm_audio::{GraphState, wasm_audio},
};

pub mod audio_graph;
pub mod command_queue;
pub mod dsp;
pub mod event_queue;
pub mod graph;
pub mod latest_value;
pub mod node_data;
pub mod nodes;
mod wasm_audio;

#[wasm_bindgen(js_name = startAudio)]
pub async fn start_audio(ctx: AudioContext) -> Result<AudioGraph, JsValue> {
    console_error_panic_hook::set_once();
    console_log::init().map_err(|e| e.to_string())?;

    let sample_rate = ctx.sample_rate() as u32;
    let (event_sender, shared_port) = event_channel();
    let (lv_writer, lv_reader) = latest_value_channel();
    let (nd_writer, nd_reader) = node_data_channel();
    let mut graph = Graph::new_with_events(sample_rate, event_sender);
    let output_node_info = graph.add::<Passthrough>();
    let output_port = graph
        .port_info(output_node_info.node_id(), 0, PortType::Input)
        .unwrap();

    let state = GraphState {
        graph,
        lv_writer,
        nd_writer,
        node_reg: NodeRegistry::initialize(),
        sample_rate,
        output_port,
    };

    let worklet_node = wasm_audio(ctx, state, shared_port)?;

    Ok(AudioGraph::new(
        lv_reader,
        nd_reader,
        output_node_info,
        NodeRegistry::initialize(),
        worklet_node,
    ))
}
