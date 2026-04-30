use core::f32;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    node::{Effect, Source},
    oscillator::Oscillator,
};

mod gain;
mod node;
mod oscillator;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    println!(
        "Default output device: {:?}",
        device.description().unwrap().name()
    );

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);

    let osc = Arc::new(Mutex::new(Oscillator::new(440.0, config.sample_rate())));
    let gain = Arc::new(Mutex::new(gain::Gain::default()));
    let ramp = Arc::new(Mutex::new(gain::Ramp::new(1.0, 0.0, 1000)));

    let osc_arc = Arc::clone(&osc);
    let gain_arc = Arc::clone(&gain);
    let ramp_arc = Arc::clone(&ramp);

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut ramp = ramp_arc.lock().unwrap();
                let mut osc = osc_arc.lock().unwrap();
                let mut gain = gain_arc.lock().unwrap();
                gain.set_gain(ramp.update());
                for sample in data.iter_mut() {
                    *sample = gain.process(osc.output());
                }
            },
            move |err| {
                eprintln!("Stream error: {:?}", err);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1000));
    ramp.lock().unwrap().start();
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
