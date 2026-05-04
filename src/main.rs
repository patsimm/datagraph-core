use core::f32;
use std::{sync::Arc, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    helpers::FromHz,
    node::{Effect, Source},
    oscillator::Oscillator,
};

mod event_buffer;
mod gain;
mod helpers;
mod node;
mod oscillator;
mod ring_buffer;

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

    let mut osc = Oscillator::new(Duration::from_hz(220.0), config.sample_rate());
    let mut gain = gain::Gain::default();
    let mut adsr = gain::ADSR::new(
        config.sample_rate(),
        std::time::Duration::from_millis(20),
        std::time::Duration::from_millis(20),
        0.7,
        std::time::Duration::from_millis(50),
    );

    gain.set_gain(0.0);

    let event_buffer = Arc::new(event_buffer::EventBuffer::new());
    let event_buffer_clone = event_buffer.clone();

    let mut i = 0;

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Some(event) = event_buffer.clone().pop() {
                    match event {
                        event_buffer::Event::NoteOn => {
                            adsr.start(i + 1);
                        }
                        event_buffer::Event::NoteOff => {
                            adsr.stop(i + 1);
                        }
                    }
                }

                for sample in data.iter_mut() {
                    i += 1;
                    if let Some(new_val) = adsr.update(i) {
                        gain.set_gain(new_val);
                    }
                    *sample = gain.process(osc.output(i), i);
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

    for _ in 0..16 {
        event_buffer_clone.push(event_buffer::Event::NoteOn);
        std::thread::sleep(std::time::Duration::from_millis(100));
        event_buffer_clone.push(event_buffer::Event::NoteOff);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
