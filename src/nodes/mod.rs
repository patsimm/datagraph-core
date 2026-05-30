mod adsr;
mod clock;
mod delay;
mod filter;
mod math;
mod monoshot;
mod noise;
mod oscillator;
mod sequencer;

use crate::graph::{CreateNode, GraphNode, Node, NodeId};
use std::collections::HashMap;

pub use crate::graph::{Param, ParamHandle};

pub struct NodeRegistry {
    registry: HashMap<&'static str, fn(NodeId, u32) -> GraphNode>,
}

impl NodeRegistry {
    pub fn register<T: CreateNode>(&mut self) {
        self.registry.insert(std::any::type_name::<T>(), T::create);
    }

    pub fn node_types(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.registry.keys().copied()
    }

    pub fn create(&self, node_id: NodeId, typename: &str, sample_rate: u32) -> Option<GraphNode> {
        self.registry
            .get(typename)
            .map(|factory| factory(node_id, sample_rate))
    }
}

macro_rules! register_nodes {
    ($($t:ty),+) => {
        $(
            pub use $t;
            impl CreateNode for $t {
                fn create(node_id: NodeId, sample_rate: u32) -> GraphNode {
                    GraphNode::new(node_id, Self::new(sample_rate))
                }
            }
        )+

        impl NodeRegistry {
            pub fn initialize() -> Self {
                let mut registry = Self {
                    registry: HashMap::new(),
                };
                $(registry.register::<$t>();)+
                registry
            }
        }
    };
}

register_nodes!(
    math::Add,
    adsr::ADSR,
    delay::Delay,
    filter::OnePoleLowPass,
    math::Multiply,
    math::Passthrough,
    sequencer::Sequencer,
    oscillator::Sin,
    oscillator::Saw,
    oscillator::Square,
    noise::Noise,
    math::Min,
    math::Max,
    clock::Clock,
    clock::ClockDivider,
    monoshot::Monoshot
);
