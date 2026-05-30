use wasm_bindgen::prelude::*;
use web_sys::AudioContext;

use crate::{
    audio_graph::AudioGraph,
    command_queue::spsc_command_queue,
    event_queue::event_channel,
    graph::{BatchTickable, Graph},
    latest_value::latest_value_channel,
    node_data::node_data_channel,
    nodes::{NodeRegistry, Passthrough},
    wasm_audio::wasm_audio,
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
    let (sender, receiver) = spsc_command_queue();
    let (event_sender, shared_port) = event_channel();
    let (mut lv_writer, lv_reader) = latest_value_channel();
    let (mut nd_writer, nd_reader) = node_data_channel();
    let mut graph = Graph::new_with_events(sample_rate, event_sender);
    let output_node_info = graph.add::<Passthrough>();
    let output_port = graph
        .port_info(output_node_info.node_id(), 0, graph::PortType::Input)
        .unwrap();
    let node_registry = NodeRegistry::initialize();

    let worklet_node = wasm_audio(
        ctx,
        Box::new(move |buf| {
            receiver.drain_into(&mut graph, &mut lv_writer, &mut nd_writer);

            let nd_ports = nd_writer.subscribed_ports();
            let mut output_ports = vec![output_port.key()];
            output_ports.extend(nd_ports.iter());

            let mut out = graph.tick_batch(&output_ports, buf.len());
            let first = out.next().unwrap_or(&[]);
            let len = first.len().min(buf.len());
            buf[..len].copy_from_slice(&first[..len]);

            nd_writer.write_batches(out);
            lv_writer.write_from_graph(&graph);
            true
        }),
        shared_port,
    )?;

    Ok(AudioGraph::new(
        sample_rate,
        sender,
        lv_reader,
        nd_reader,
        output_node_info,
        node_registry,
        worklet_node,
    ))
}
