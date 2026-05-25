pub mod add;
pub mod adsr;
pub mod delay;
pub mod filter;
pub mod multiply;
pub mod oscillator;
pub mod param;
pub mod passthrough;
pub mod sequencer;

use crate::graph::{CreateNode, GraphNode, Node};
use std::collections::HashMap;
use std::sync::OnceLock;

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
            impl CreateNode for $t {
                fn create(sample_rate: u32) -> GraphNode {
                    GraphNode::from(Self::new(sample_rate))
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
    add::Add,
    adsr::ADSR,
    delay::Delay,
    filter::OnePoleLowPass,
    multiply::Multiply,
    passthrough::Passthrough,
    sequencer::Sequencer,
    oscillator::Sin,
    oscillator::Saw,
    oscillator::Square
);
