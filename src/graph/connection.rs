use super::port::PortIndex;

pub(super) struct Connection {
    pub(super) from_idx: usize,
    pub(super) from_port: PortIndex,
    pub(super) to_idx: usize,
    pub(super) to_port: PortIndex,
}
