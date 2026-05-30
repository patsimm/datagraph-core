use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, str::FromStr};
use tsify_next::Tsify;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct NodeId(#[tsify(type = "string")] [char; 8]);

impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.0.iter().collect();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NodeId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for NodeId {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        NodeId::from_str(&s)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.iter().collect();
        write!(f, "{}", s)
    }
}

impl FromStr for NodeId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 8 {
            return Err(format!("NodeId must be 8 characters long, got {}", s.len()));
        }
        let chars = s.chars().collect::<Vec<_>>();
        chars
            .try_into()
            .map(NodeId)
            .map_err(|_| "Failed to parse NodeId".to_string())
    }
}

impl NodeId {
    pub fn new() -> Self {
        NodeId(nanoid!(8).chars().collect::<Vec<_>>().try_into().unwrap())
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
