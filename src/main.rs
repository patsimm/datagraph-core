use core::f32;
use std::{sync::Arc, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    helpers::FromHz,
    node::{Effect, Source},
    oscillator::Oscillator,
    param::Param,
};

mod delay;
mod event_buffer;
mod gain;
mod helpers;
mod node;
mod oscillator;
mod param;
mod ring_buffer;
mod wav;

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
    let mut adsr = gain::ADSR::new(
        config.sample_rate(),
        std::time::Duration::from_millis(50),
        std::time::Duration::from_millis(20),
        0.7,
        std::time::Duration::from_millis(150),
    );
    let mut delay = delay::Delay::new();
    let mut gain = gain::Gain {
        param: Param::from(0.1),
    };

    // let samples = (0..44100 * 5)
    //     .map(|i| {
    //         if i == 5000 {
    //             adsr.start(i);
    //         }
    //         if i == 22050 + 5000 {
    //             adsr.stop(i);
    //         }
    //         adsr.process(osc.output(i), i)
    //     })
    //     .map(|s| (s * i16::MAX as f32) as i16)
    //     .collect::<Vec<_>>();
    // write_wav("output.wav", &samples, config.sample_rate());

    let event_buffer = Arc::new(event_buffer::EventBuffer::new());
    let event_buffer_clone = event_buffer.clone();

    let mut i = 0;

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Some(event) = event_buffer.clone().pop() {
                    match event {
                        event_buffer::Event::NoteOn { frequency } => {
                            osc.frequency.set(frequency);
                            adsr.start(i + 1);
                        }
                        event_buffer::Event::NoteOff => {
                            adsr.stop(i + 1);
                        }
                    }
                }

                for sample in data.iter_mut() {
                    i += 1;
                    *sample = delay.process(adsr.process(osc.output(i), i), i);
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
    for i in 0..16 {
        event_buffer_clone
            .push(event_buffer::Event::NoteOn {
                frequency: Duration::from_hz(i as f32 * 220.0 / 8.0 + 110.0).as_secs_f32(),
            })
            .expect("Failed to push event");
        std::thread::sleep(std::time::Duration::from_millis(100));
        event_buffer_clone
            .push(event_buffer::Event::NoteOff)
            .expect("Failed to push event");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
