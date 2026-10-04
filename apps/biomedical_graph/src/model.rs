use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub type Properties = BTreeMap<String, Value>;

/// All domain nodes also receive GraphNode and a globally unique uid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub uid: String,
    pub labels: Vec<String>,
    pub properties: Properties,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub uid: String,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub properties: Properties,
}
/// Open extension point: add new node labels and relationship types without changing an enum.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphBatch {
    pub nodes: BTreeMap<String, Node>,
    pub edges: BTreeMap<String, Edge>,
}
impl GraphBatch {
    pub fn node(&mut self, uid: impl Into<String>, label: &str, properties: Properties) -> String {
        let uid = uid.into();
        let node = self.nodes.entry(uid.clone()).or_insert_with(|| Node {
            uid: uid.clone(),
            labels: Vec::new(),
            properties: Properties::new(),
        });
        if !node.labels.iter().any(|l| l == label) {
            node.labels.push(label.into());
        }
        node.properties.extend(properties);
        uid
    }
    /// Identity includes endpoints, type, and caller-provided provenance key.
    pub fn edge(
        &mut self,
        source: &str,
        kind: &str,
        target: &str,
        provenance: &str,
        properties: Properties,
    ) {
        let uid = hash(&(source, kind, target, provenance));
        self.edges.insert(
            uid.clone(),
            Edge {
                uid,
                source: source.into(),
                target: target.into(),
                kind: kind.into(),
                properties,
            },
        );
    }
    pub fn extend(&mut self, other: Self) {
        for node in other.nodes.into_values() {
            for label in node.labels {
                self.node(&node.uid, &label, node.properties.clone());
            }
        }
        self.edges.extend(other.edges);
    }
    pub(crate) fn validate(&self) -> Result<()> {
        for node in self.nodes.values() {
            if node.uid.is_empty() || node.labels.is_empty() {
                return Err(Error::Invalid("empty node uid or labels".into()));
            }
            for label in &node.labels {
                identifier(label)?;
            }
            if node.properties.contains_key("uid") {
                return Err(Error::Invalid("uid is reserved".into()));
            }
        }
        for edge in self.edges.values() {
            identifier(&edge.kind)?;
            if !self.nodes.contains_key(&edge.source) || !self.nodes.contains_key(&edge.target) {
                return Err(Error::Invalid(format!(
                    "missing endpoint for {}",
                    edge.kind
                )));
            }
            if edge.properties.contains_key("uid") {
                return Err(Error::Invalid("uid is reserved".into()));
            }
        }
        Ok(())
    }
}
pub(crate) fn identifier(value: &str) -> Result<()> {
    let mut chars = value.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(Error::Invalid(format!(
            "unsafe Cypher identifier {value:?}"
        )));
    }
    Ok(())
}
pub(crate) fn hash(value: &impl Serialize) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("serializable graph identity"))
    )
}
pub(crate) fn props(value: impl Serialize) -> Properties {
    match serde_json::to_value(value).expect("serializable graph properties") {
        Value::Object(map) => map.into_iter().collect(),
        value => BTreeMap::from([("value".into(), value)]),
    }
}
