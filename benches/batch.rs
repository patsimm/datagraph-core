use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use datagraph::graph::PortKey;
use datagraph::graph::{BatchTickable, Graph, PortType};
use datagraph::nodes::Add;

fn setup_graph() -> (Graph, Vec<PortKey>) {
    let mut graph = Graph::new(44100);
    let p1 = graph.add_param(0.5);
    let p2 = graph.add_param(0.3);
    let add = graph.add::<Add>();
    graph.connect(p1, 0, add, 0).unwrap();
    graph.connect(p2, 0, add, 1).unwrap();
    let outputs = vec![PortKey {
        node_id: add,
        port_index: 0,
        port_type: PortType::Output,
    }];
    (graph, outputs)
}

fn bench_tick_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_batch");
    for size in [64, 128, 256, 512, 1024] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let (mut graph, outputs) = setup_graph();
            b.iter(|| {
                graph
                    .tick_batch(black_box(&outputs), black_box(size))
                    .count()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tick_batch_sizes);
criterion_main!(benches);
