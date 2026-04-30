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
    let adsr = Arc::new(Mutex::new(gain::ADSR::new(
        std::time::Duration::from_millis(50),
        std::time::Duration::from_millis(50),
        0.7,
        std::time::Duration::from_millis(50),
    )));

    let osc_arc = Arc::clone(&osc);
    let gain_arc = Arc::clone(&gain);
    let adsr_arc = Arc::clone(&adsr);

    gain.lock().unwrap().set_gain(0.0);

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut adsr = adsr_arc.lock().unwrap();
                let mut osc = osc_arc.lock().unwrap();
                let mut gain = gain_arc.lock().unwrap();
                if let Some(new_val) = adsr.update() {
                    gain.set_gain(new_val);
                }
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
    for _ in 0..100 {
        adsr.lock().unwrap().start();
        std::thread::sleep(std::time::Duration::from_millis(200));
        adsr.lock().unwrap().stop();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    std::thread::sleep(std::time::Duration::from_millis(10000));
}
