use std::panic::AssertUnwindSafe;

use tsify_next::declare;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions};

use crate::event_queue::SharedPort;

#[declare]
type ProcessFn = dyn FnMut(&mut [f32]) -> bool;

#[wasm_bindgen]
pub struct WasmAudioProcessor {
    process_fn: Box<ProcessFn>,
    port: SharedPort,
}

impl WasmAudioProcessor {
    pub fn new(process_fn: Box<ProcessFn>, port: SharedPort) -> Self {
        Self { process_fn, port }
    }
}

#[wasm_bindgen]
impl WasmAudioProcessor {
    pub fn process(&mut self, buf: &mut [f32]) -> bool {
        (self.process_fn)(buf)
    }

    #[wasm_bindgen(js_name = setPort)]
    pub fn set_port(&mut self, port: web_sys::MessagePort) {
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
    process_fn: Box<ProcessFn>,
    port: SharedPort,
) -> Result<AudioWorkletNode, JsValue> {
    let processor = AssertUnwindSafe(WasmAudioProcessor::new(process_fn, port));
    let node = wasm_audio_node(&ctx, processor.0)?;
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
