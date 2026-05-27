use super::NodeId;

pub(super) struct Connection {
    pub(super) from: NodeId,
    pub(super) from_port: usize,
    pub(super) to: NodeId,
    pub(super) to_port: usize,
}
