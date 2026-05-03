use core::f32;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    node::{Effect, Source},
    oscillator::Oscillator,
};

mod event_buffer;
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

    let mut osc = Oscillator::new(440.0, config.sample_rate());
    let mut gain = gain::Gain::default();
    let mut adsr = gain::ADSR::new(
        std::time::Duration::from_millis(100),
        std::time::Duration::from_millis(50),
        0.7,
        std::time::Duration::from_millis(100),
    );

    gain.set_gain(0.0);
    let time_per_sample = Duration::from_secs_f32(1.0 / config.sample_rate() as f32);

    let event_buffer = Arc::new(event_buffer::EventBuffer::<16>::new());
    let event_buffer_clone = event_buffer.clone();

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let now = Instant::now();

                while let Some(event) = event_buffer.clone().pop() {
                    match event {
                        event_buffer::Event::NoteOn { note, velocity } => {
                            adsr.start();
                        }
                        event_buffer::Event::NoteOff { note: _ } => {
                            adsr.stop();
                        }
                    }
                }

                for (i, sample) in data.iter_mut().enumerate() {
                    if let Some(new_val) = adsr.update(now + time_per_sample * i as u32) {
                        gain.set_gain(new_val);
                    }
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
    event_buffer_clone.push(event_buffer::Event::NoteOn {
        note: 60,
        velocity: 1.0,
    });
    std::thread::sleep(std::time::Duration::from_millis(1000));
    event_buffer_clone.push(event_buffer::Event::NoteOff { note: 60 });

    std::thread::sleep(std::time::Duration::from_millis(10000));
}
