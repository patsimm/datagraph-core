use core::f32;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use datagraph::{
    frequency::{Frequency, ToCv},
    graph::{Graph, GraphNode, PortType, PortValueAccess, Tickable},
    nodes::{ADSR, Delay, Multiply, Param, Sin},
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

    let freq_param = Param::from(Frequency::from(Note::C4).to_cv());
    let mut freq_handle = freq_param.handle();

    let gate_param = Param::from(0.0_f32);
    let mut gate_handle = gate_param.handle();

    let mut graph = Graph::new(sample_rate);

    let freq_node = graph.add_node(GraphNode::new(freq_param));
    let osc_node = graph.add::<Sin>();
    graph
        .connect(freq_node, 0, osc_node, 0)
        .expect("Failed to connect frequency to oscillator");

    let adsr_gate_node = graph.add_node(GraphNode::new(gate_param));
    let attack_node = graph.add_param(0.05);
    let decay_node = graph.add_param(0.02);
    let sustain_node = graph.add_param(0.7);
    let release_node = graph.add_param(0.15);
    let adsr_node = graph.add::<ADSR>();
    let adsr_gain = graph.add::<Multiply>();
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

    let delay_node = graph.add::<Delay>();
    let gain_value = graph.add_param(0.5);
    let gain_node = graph.add::<Multiply>();
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
        freq_handle.set(Frequency::from(Note::from(notes[i % notes.len()])).to_cv());
        gate_handle.set(1.0);
        std::thread::sleep(std::time::Duration::from_millis(200));
        gate_handle.set(0.0);
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
