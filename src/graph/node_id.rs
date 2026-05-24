use nanoid::nanoid;
use std::{fmt::Display, ops::Deref, str::FromStr};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId([char; 8]);

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId::from_str(&s).unwrap_or(NodeId::invalid())
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.iter().collect();
        write!(f, "{}", s)
    }
}

impl FromStr for NodeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 8 {
            return Err(());
        }
        let chars = s.chars().collect::<Vec<_>>();
        chars.try_into().map(NodeId).map_err(|_| ())
    }
}

impl NodeId {
    pub fn new() -> Self {
        NodeId(nanoid!(8).chars().collect::<Vec<_>>().try_into().unwrap())
    }

    pub fn invalid() -> Self {
        NodeId(['\0'; 8])
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for NodeId {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
