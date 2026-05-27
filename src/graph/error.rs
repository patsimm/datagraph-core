use std::fmt::Display;

use super::node_id::NodeId;
use super::port::PortType;

#[derive(Debug)]
pub enum GraphError {
    NodeNotFound {
        node_id: NodeId,
    },
    NotAParameter {
        node_id: NodeId,
    },
    PortNotFound {
        node_id: NodeId,
        port: usize,
        port_type: PortType,
    },
    PortAlreadyConnected {
        node_id: NodeId,
        port: usize,
        port_type: PortType,
    },
    ImpossibleConnection {
        from_node_id: NodeId,
        from_port: usize,
        to_node_id: NodeId,
        to_port: usize,
    },
    InvalidPortKey {
        key: String,
    },
    InvalidNodeId {
        id: String,
    },
}

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::NodeNotFound { node_id } => write!(f, "Node not found: {:?}", *node_id)?,
            GraphError::NotAParameter { node_id } => {
                write!(f, "Node is not a parameter: {:?}", *node_id)?
            }
            GraphError::PortNotFound {
                node_id,
                port,
                port_type,
            } => {
                let port_type_str = match port_type {
                    PortType::Input => "input",
                    PortType::Output => "output",
                };
                write!(
                    f,
                    "Port not found: node {:?} has no {} port {}",
                    node_id, port_type_str, port
                )?
            }
            GraphError::PortAlreadyConnected {
                node_id,
                port,
                port_type,
            } => {
                let port_type_str = match port_type {
                    PortType::Input => "input",
                    PortType::Output => "output",
                };
                write!(
                    f,
                    "Port already connected: node {:?} {} port {} is already connected",
                    node_id, port_type_str, port
                )?
            }
            GraphError::ImpossibleConnection {
                from_node_id,
                from_port,
                to_node_id,
                to_port,
            } => write!(
                f,
                "Impossible connection: cannot connect output port {} of node {:?} to input port {} of node {:?}",
                from_port, from_node_id, to_port, to_node_id
            )?,
            GraphError::InvalidPortKey { key } => write!(f, "Invalid port key: {}", key)?,
            GraphError::InvalidNodeId { id } => write!(f, "Invalid node id: {}", id)?,
        };
        Ok(())
    }
}
