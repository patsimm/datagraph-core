use core::f32;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    delay::Delay,
    frequency::{Frequency, ToCv},
    gain::{ADSR, Gain},
    note::Note,
    oscillator::Oscillator,
    param::Param,
};

mod delay;
mod event_buffer;
mod frequency;
mod gain;
mod graph;
mod helpers;
mod note;
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

    let mut freq = Param::from(Frequency::from(Note::C4).to_cv());
    let osc = Oscillator::new(config.sample_rate());
    let mut adsr_gate = Param::from(0.0);
    let adsr = ADSR::new(
        config.sample_rate(),
        std::time::Duration::from_millis(50),
        std::time::Duration::from_millis(20),
        0.7,
        std::time::Duration::from_millis(150),
    );
    let delay = Delay::new();
    let gain = Gain;

    let mut graph = graph::Graph::new();
    let freq_node = graph.add_node(freq.node());
    let osc_node = graph.add_node(osc);
    graph.connect(freq_node, 0, osc_node, 0);

    let adsr_gate_node = graph.add_node(adsr_gate.node());
    let adsr_node = graph.add_node(adsr);
    let adsr_gain = graph.add_node(Gain);
    graph.connect(adsr_gate_node, 0, adsr_node, 0);
    graph.connect(adsr_node, 0, adsr_gain, 1);

    let delay_node = graph.add_node(delay);

    let gain_value = graph.add_node(Param::new(0.5).node());
    let gain_node = graph.add_node(gain);
    graph.connect(gain_value, 0, gain_node, 1);

    graph.connect(osc_node, 0, adsr_gain, 0);
    graph.connect(adsr_gain, 0, delay_node, 0);
    graph.connect(delay_node, 0, gain_node, 0);

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

    let mut i = 0;
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    i += 1;
                    graph.tick(i);
                    *sample = graph.output(gain_node)[0];
                }
            },
            move |err| {
                eprintln!("Stream error: {:?}", err);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    let notes = &["C4", "D4", "E4", "D4"];

    std::thread::sleep(std::time::Duration::from_millis(1000));
    for i in 0..16 {
        // event_buffer_clone
        //     .push(event_buffer::Event::NoteOn {
        //         frequency: *Note::from(notes[i % notes.len()]).midi().to_frequency(),
        //     })
        //     .expect("Failed to push event");
        freq.set(Frequency::from(Note::from(notes[i % notes.len()])).to_cv());
        adsr_gate.set(1.0);
        std::thread::sleep(std::time::Duration::from_millis(200));
        // event_buffer_clone
        //     .push(event_buffer::Event::NoteOff)
        //     .expect("Failed to push event");
        adsr_gate.set(0.0);
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
