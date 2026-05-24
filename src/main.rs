use core::f32;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use datagraph::{
    frequency::{Frequency, ToCv},
    graph::{Graph, Multiply, Node, PortType},
    nodes::{adsr::ADSR, delay::Delay, oscillator::Sin, param::Param},
    note::Note,
};

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

    let sample_rate = config.sample_rate();

    let mut freq = Param::from(Frequency::from(Note::C4).to_cv());
    let osc = Sin::new(sample_rate);
    let mut adsr_gate = Param::from(0.0_f32);
    let adsr = ADSR::new(sample_rate);
    let attack_param = Param::new(0.05_f32);
    let decay_param = Param::new(0.02_f32);
    let sustain_param = Param::new(0.7_f32);
    let release_param = Param::new(0.15_f32);
    let delay = Delay::new(sample_rate);
    let gain = Multiply;

    let mut graph = Graph::new();
    let freq_node = graph.add(freq.node());
    let osc_node = graph.add(osc);
    graph
        .connect(freq_node, 0, osc_node, 0)
        .expect("Failed to connect frequency to oscillator");

    let adsr_gate_node = graph.add(adsr_gate.node());
    let attack_node = graph.add(attack_param.node());
    let decay_node = graph.add(decay_param.node());
    let sustain_node = graph.add(sustain_param.node());
    let release_node = graph.add(release_param.node());
    let adsr_node = graph.add(adsr);
    let adsr_gain = graph.add(Multiply);
    graph
        .connect(adsr_gate_node, 0, adsr_node, 0)
        .expect("Failed to connect ADSR gate to ADSR");
    graph
        .connect(attack_node, 0, adsr_node, 1)
        .expect("Failed to connect attack to ADSR");
    graph
        .connect(decay_node, 0, adsr_node, 2)
        .expect("Failed to connect decay to ADSR");
    graph
        .connect(sustain_node, 0, adsr_node, 3)
        .expect("Failed to connect sustain to ADSR");
    graph
        .connect(release_node, 0, adsr_node, 4)
        .expect("Failed to connect release to ADSR");
    graph
        .connect(adsr_node, 0, adsr_gain, 1)
        .expect("Failed to connect ADSR to ADSR gain");

    let delay_node = graph.add(delay);

    let gain_value = graph.add(Param::new(0.5).node());
    let gain_node = graph.add(gain);
    graph
        .connect(gain_value, 0, gain_node, 1)
        .expect("Failed to connect gain value to gain node");

    graph
        .connect(osc_node, 0, adsr_gain, 0)
        .expect("Failed to connect oscillator to ADSR gain");
    graph
        .connect(adsr_gain, 0, delay_node, 0)
        .expect("Failed to connect ADSR gain to delay");
    graph
        .connect(delay_node, 0, gain_node, 0)
        .expect("Failed to connect delay to gain");

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    graph.tick();
                    *sample = *graph
                        .port_value(gain_node, 0, PortType::Output)
                        .unwrap_or(&0.0);
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
        freq.set(Frequency::from(Note::from(notes[i % notes.len()])).to_cv());
        adsr_gate.set(1.0);
        std::thread::sleep(std::time::Duration::from_millis(200));
        adsr_gate.set(0.0);
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
