use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::{
    graph::{Graph, GraphNode, NodeId, PortKey},
    latest_value::LatestValueWriter,
    node_data::NodeDataWriter,
};

pub enum GraphCommand {
    AddNode {
        id: NodeId,
        node: GraphNode,
    },
    AddParam {
        id: NodeId,
        value: f32,
    },
    RemoveNode(NodeId),
    Connect {
        from: NodeId,
        from_port: usize,
        to: NodeId,
        to_port: usize,
    },
    Disconnect {
        from: NodeId,
        from_port: usize,
        to: NodeId,
        to_port: usize,
    },
    SetParamValue {
        id: NodeId,
        value: f32,
    },
    SetDefaultInputValue {
        id: NodeId,
        port: usize,
        value: f32,
    },
    SubscribeLatestValue {
        port: PortKey,
        index: usize,
    },
    UnsubscribeLatestValue {
        index: usize,
    },
    SubscribeNodeData {
        port: PortKey,
        index: usize,
    },
    UnsubscribeNodeData {
        index: usize,
    },
}

const CAPACITY: usize = 64;

struct SpscInner<T> {
    slots: Box<[UnsafeCell<MaybeUninit<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl<T: Send> Send for SpscInner<T> {}
unsafe impl<T: Send> Sync for SpscInner<T> {}

impl<T> SpscInner<T> {
    fn new() -> Self {
        let mut slots = Vec::with_capacity(CAPACITY);
        for _ in 0..CAPACITY {
            slots.push(UnsafeCell::new(MaybeUninit::uninit()));
        }
        Self {
            slots: slots.into_boxed_slice(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    fn try_push(&self, value: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % CAPACITY;
        if next_tail == self.head.load(Ordering::Acquire) {
            return Err(value);
        }
        unsafe { (*self.slots[tail].get()).write(value) };
        self.tail.store(next_tail, Ordering::Release);
        Ok(())
    }

    fn try_pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        if head == self.tail.load(Ordering::Acquire) {
            return None;
        }
        let value = unsafe { (*self.slots[head].get()).assume_init_read() };
        self.head.store((head + 1) % CAPACITY, Ordering::Release);
        Some(value)
    }
}

impl<T> Drop for SpscInner<T> {
    fn drop(&mut self) {
        while self.try_pop().is_some() {}
    }
}

pub struct CommandSender(Arc<SpscInner<GraphCommand>>);
pub struct CommandReceiver(Arc<SpscInner<GraphCommand>>);

pub fn spsc_command_queue() -> (CommandSender, CommandReceiver) {
    let inner = Arc::new(SpscInner::new());
    (CommandSender(Arc::clone(&inner)), CommandReceiver(inner))
}

impl CommandSender {
    pub fn add_node(&self, node: GraphNode) -> NodeId {
        let id = NodeId::new();
        let _ = self.0.try_push(GraphCommand::AddNode { id, node });
        id
    }

    pub fn add_node_with_id(&self, id: NodeId, node: GraphNode) {
        let _ = self.0.try_push(GraphCommand::AddNode { id, node });
    }

    pub fn add_param(&self, value: f32) -> NodeId {
        let id = NodeId::new();
        let _ = self.0.try_push(GraphCommand::AddParam { id, value });
        id
    }

    pub fn add_param_with_id(&self, id: NodeId, value: f32) {
        let _ = self.0.try_push(GraphCommand::AddParam { id, value });
    }

    pub fn remove_node(&self, id: NodeId) {
        let _ = self.0.try_push(GraphCommand::RemoveNode(id));
    }

    pub fn connect(&self, from: NodeId, from_port: usize, to: NodeId, to_port: usize) {
        let _ = self.0.try_push(GraphCommand::Connect {
            from,
            from_port,
            to,
            to_port,
        });
    }

    pub fn disconnect(&self, from: NodeId, from_port: usize, to: NodeId, to_port: usize) {
        let _ = self.0.try_push(GraphCommand::Disconnect {
            from,
            from_port,
            to,
            to_port,
        });
    }

    pub fn set_param_value(&self, id: NodeId, value: f32) {
        let _ = self.0.try_push(GraphCommand::SetParamValue { id, value });
    }

    pub fn set_default_input_value(&self, id: NodeId, port: usize, value: f32) {
        let _ = self
            .0
            .try_push(GraphCommand::SetDefaultInputValue { id, port, value });
    }

    pub fn subscribe_latest_value(&self, port: PortKey, index: usize) {
        let _ = self
            .0
            .try_push(GraphCommand::SubscribeLatestValue { port, index });
    }

    pub fn unsubscribe_latest_value(&self, index: usize) {
        let _ = self
            .0
            .try_push(GraphCommand::UnsubscribeLatestValue { index });
    }

    pub fn subscribe_node_data(&self, port: PortKey, index: usize) {
        let _ = self
            .0
            .try_push(GraphCommand::SubscribeNodeData { port, index });
    }

    pub fn unsubscribe_node_data(&self, index: usize) {
        let _ = self
            .0
            .try_push(GraphCommand::UnsubscribeNodeData { index });
    }
}

impl CommandReceiver {
    pub fn drain_into(
        &self,
        graph: &mut Graph,
        lv_writer: &mut LatestValueWriter,
        nd_writer: &mut NodeDataWriter,
    ) {
        while let Some(cmd) = self.0.try_pop() {
            apply_command(cmd, graph, lv_writer, nd_writer);
        }
    }
}

fn apply_command(
    cmd: GraphCommand,
    graph: &mut Graph,
    lv_writer: &mut LatestValueWriter,
    nd_writer: &mut NodeDataWriter,
) {
    match cmd {
        GraphCommand::AddNode { id, node } => {
            graph.insert_node(id, node);
        }
        GraphCommand::AddParam { id, value } => {
            graph.add_param_with_id(id, value);
        }
        GraphCommand::RemoveNode(id) => {
            let _ = graph.remove_node(&id);
        }
        GraphCommand::Connect {
            from,
            from_port,
            to,
            to_port,
        } => {
            let _ = graph.connect(&from, from_port, &to, to_port);
        }
        GraphCommand::Disconnect {
            from,
            from_port,
            to,
            to_port,
        } => {
            let _ = graph.disconnect(&from, from_port, &to, to_port);
        }
        GraphCommand::SetParamValue { id, value } => {
            let _ = graph.set_param_value(&id, value);
        }
        GraphCommand::SetDefaultInputValue { id, port, value } => {
            let _ = graph.set_default_input_value(&id, port, value);
        }
        GraphCommand::SubscribeLatestValue { port, index } => lv_writer.subscribe(port, index),
        GraphCommand::UnsubscribeLatestValue { index } => lv_writer.unsubscribe(index),
        GraphCommand::SubscribeNodeData { port, index } => nd_writer.subscribe(port, index),
        GraphCommand::UnsubscribeNodeData { index } => nd_writer.unsubscribe(index),
    }
}
