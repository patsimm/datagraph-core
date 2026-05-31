use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, MessageEvent};

use crate::{
    command_queue::{apply_command, GraphCommand},
    event_queue::SharedPort,
    graph::{BatchTickable, Graph, PortInfo, PortKey},
    latest_value::LatestValueWriter,
    node_data::NodeDataWriter,
    nodes::NodeRegistry,
};

pub(crate) struct GraphState {
    pub graph: Graph,
    pub lv_writer: LatestValueWriter,
    pub nd_writer: NodeDataWriter,
    pub node_reg: NodeRegistry,
    pub sample_rate: u32,
    pub output_port: PortInfo,
}

#[wasm_bindgen]
pub struct WasmAudioProcessor {
    state: Rc<RefCell<GraphState>>,
    port: SharedPort,
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    output_ports: Vec<PortKey>,
    nd_version: u64,
}

impl WasmAudioProcessor {
    pub(crate) fn new(state: Rc<RefCell<GraphState>>, port: SharedPort) -> Self {
        Self {
            state,
            port,
            _onmessage: None,
            output_ports: Vec::new(),
            nd_version: u64::MAX,
        }
    }
}

#[wasm_bindgen]
impl WasmAudioProcessor {
    pub fn process(&mut self, buf: &mut [f32]) -> bool {
        let mut guard = self.state.borrow_mut();
        let s = &mut *guard;

        let v = s.nd_writer.version();
        if v != self.nd_version || self.output_ports.is_empty() {
            self.output_ports.clear();
            self.output_ports.push(*s.output_port.key());
            self.output_ports
                .extend(s.nd_writer.subscriptions().iter().map(|(p, _)| *p));
            self.nd_version = v;
        }

        let mut out = s.graph.tick_batch(&self.output_ports, buf.len());
        let first = out.next().unwrap_or(&[]);
        let len = first.len().min(buf.len());
        buf[..len].copy_from_slice(&first[..len]);

        s.nd_writer.write_batches(out);
        s.lv_writer.write_from_graph(&s.graph);
        true
    }

    #[wasm_bindgen(js_name = setPort)]
    pub fn set_port(&mut self, port: web_sys::MessagePort) {
        let state = Rc::clone(&self.state);
        let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Ok(cmd) = serde_wasm_bindgen::from_value::<GraphCommand>(e.data()) {
                let mut guard = state.borrow_mut();
                let s = &mut *guard;
                apply_command(
                    cmd,
                    &mut s.graph,
                    &mut s.lv_writer,
                    &mut s.nd_writer,
                    &s.node_reg,
                    s.sample_rate,
                );
            }
        });
        port.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        self._onmessage = Some(closure);
        *self.port.borrow_mut() = Some(port);
    }

    pub fn pack(self) -> usize {
        Box::into_raw(Box::new(self)) as usize
    }

    pub unsafe fn unpack(val: usize) -> Self {
        unsafe { *Box::from_raw(val as *mut _) }
    }
}

pub fn wasm_audio(
    ctx: AudioContext,
    state: GraphState,
    port: SharedPort,
) -> Result<AudioWorkletNode, JsValue> {
    let state = Rc::new(RefCell::new(state));
    let processor = WasmAudioProcessor::new(state, port);
    let node = wasm_audio_node(&ctx, processor)?;
    node.connect_with_audio_node(&ctx.destination())?;
    Ok(node)
}

pub fn wasm_audio_node(
    ctx: &AudioContext,
    processor: WasmAudioProcessor,
) -> Result<AudioWorkletNode, JsValue> {
    let options = AudioWorkletNodeOptions::new();
    options.set_processor_options(Some(&js_sys::Array::of(&[
        wasm_bindgen::module(),
        wasm_bindgen::memory(),
        processor.pack().into(),
    ])));
    AudioWorkletNode::new_with_options(ctx, "datagraph-processor", &options)
}
