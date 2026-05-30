use std::{cell::RefCell, rc::Rc};

use serde::Serialize;
use tsify_next::Tsify;

use crate::graph::{NodeInfo, PortInfo};

#[derive(Clone, Debug, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GraphEvent {
    #[serde(rename_all = "camelCase")]
    NodeAdded { node_info: NodeInfo },
    #[serde(rename_all = "camelCase")]
    NodeRemoved { node_info: NodeInfo },
    #[serde(rename_all = "camelCase")]
    Connected {
        from_port: PortInfo,
        to_port: PortInfo,
    },
    #[serde(rename_all = "camelCase")]
    Disconnected {
        from_port: PortInfo,
        to_port: PortInfo,
    },
}

pub type SharedPort = Rc<RefCell<Option<web_sys::MessagePort>>>;

pub struct EventSender(SharedPort);

pub fn event_channel() -> (EventSender, SharedPort) {
    let port: SharedPort = Rc::new(RefCell::new(None));
    (EventSender(Rc::clone(&port)), port)
}

impl EventSender {
    fn emit(&self, event: GraphEvent) {
        if let Some(port) = self.0.borrow().as_ref()
            && let Ok(val) = serde_wasm_bindgen::to_value(&event)
        {
            let _ = port.post_message(&val);
        }
    }

    pub fn push_node_added(&self, node_info: NodeInfo) {
        self.emit(GraphEvent::NodeAdded { node_info });
    }

    pub fn push_node_removed(&self, node_info: NodeInfo) {
        self.emit(GraphEvent::NodeRemoved { node_info });
    }

    pub fn push_connected(&self, from_port: PortInfo, to_port: PortInfo) {
        self.emit(GraphEvent::Connected { from_port, to_port });
    }

    pub fn push_disconnected(&self, from_port: PortInfo, to_port: PortInfo) {
        self.emit(GraphEvent::Disconnected { from_port, to_port });
    }
}
