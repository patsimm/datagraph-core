mod adsr;
mod clock;
mod delay;
mod monoshot;
mod filter;
mod math;
mod noise;
mod oscillator;
mod sequencer;

use crate::graph::{CreateNode, GraphNode, Node};
use std::collections::HashMap;
use std::sync::OnceLock;

pub use crate::graph::{Param, ParamHandle};

static INSTANCE: OnceLock<NodeRegistry> = OnceLock::new();

pub struct NodeRegistry {
    registry: HashMap<&'static str, fn(u32) -> GraphNode>,
}

impl NodeRegistry {
    pub fn global() -> &'static Self {
        INSTANCE.get_or_init(Self::initialize)
    }

    pub fn register<T: CreateNode>(&mut self) {
        self.registry.insert(std::any::type_name::<T>(), T::create);
    }

    pub fn node_types(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.registry.keys().copied()
    }

    pub fn create(&self, typename: &str, sample_rate: u32) -> Option<GraphNode> {
        self.registry
            .get(typename)
            .map(|factory| factory(sample_rate))
    }
}

macro_rules! register_nodes {
    ($($t:ty),+) => {
        $(
            pub use $t;
            impl CreateNode for $t {
                fn create(sample_rate: u32) -> GraphNode {
                    GraphNode::new(Self::new(sample_rate))
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
