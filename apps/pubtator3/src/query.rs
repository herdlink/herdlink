use crate::{EntityId, Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroU32};

macro_rules! wire_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name { $(#[serde(rename = $wire)] $variant),+ }
        impl $name {
            pub const fn as_str(self) -> &'static str { match self { $(Self::$variant => $wire),+ } }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
        }
    };
}

wire_enum!(Concept {
    Gene => "gene", Disease => "disease", Chemical => "chemical",
    Variant => "variant", Species => "species", CellLine => "cellline",
});

wire_enum!(
    /// The four entity types accepted by relation discovery.
    RelationEntityType {
    Gene => "gene", Disease => "disease", Chemical => "chemical", Variant => "variant",
});

wire_enum!(RelationType {
    Treat => "treat", Cause => "cause", Cotreat => "cotreat", Convert => "convert",
    Compare => "compare", Interact => "interact", Associate => "associate",
    PositiveCorrelate => "positive_correlate", NegativeCorrelate => "negative_correlate",
    Prevent => "prevent", Inhibit => "inhibit", Stimulate => "stimulate", DrugInteract => "drug_interact",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelationFilter {
    #[default]
    Any,
    Type(RelationType),
}

impl fmt::Display for RelationFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("ANY"),
            Self::Type(t) => t.fmt(f),
        }
    }
}

/// One-based search page. Zero is rejected, including during deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Page(NonZeroU32);

impl Page {
    pub const FIRST: Self = Self(NonZeroU32::MIN);
    pub fn new(value: u32) -> Result<Self> {
        NonZeroU32::new(value)
            .map(Self)
            .ok_or_else(|| Error::InvalidRequest("page must be positive".into()))
    }
    pub const fn get(self) -> u32 {
        self.0.get()
    }
    pub fn next(self) -> Option<Self> {
        self.get()
            .checked_add(1)
            .and_then(NonZeroU32::new)
            .map(Self)
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::FIRST
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextScope {
    #[default]
    Abstract,
    FullText,
}

wire_enum!(ExportFormat { PubTator => "pubtator", BioCXml => "biocxml", BioCJson => "biocjson" });

/// Free text or a composed PubTator entity/relation query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery(String);

impl SearchQuery {
    /// Accepts the service's complete query syntax, including field restrictions.
    pub fn text(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(Error::InvalidRequest("search text cannot be empty".into()));
        }
        Ok(Self(text))
    }
    pub fn entity(entity: &EntityId) -> Self {
        Self(entity.to_string())
    }
    pub fn relation(kind: RelationFilter, source: &EntityId, target: &EntityId) -> Self {
        Self(format!("relations:{kind}|{source}|{target}"))
    }
    pub fn relation_to_type(
        kind: RelationFilter,
        source: &EntityId,
        target: RelationEntityType,
    ) -> Self {
        Self(format!(
            "relations:{kind}|{source}|{}",
            target.as_str().to_ascii_uppercase()
        ))
    }
    pub fn and(self, other: Self) -> Self {
        Self(format!("({}) AND ({})", self.0, other.0))
    }
    pub fn or(self, other: Self) -> Self {
        Self(format!("({}) OR ({})", self.0, other.0))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct AutocompleteRequest {
    pub query: String,
    pub concept: Option<Concept>,
    pub limit: Option<NonZeroU32>,
}

impl AutocompleteRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            concept: None,
            limit: None,
        }
    }
    pub fn concept(mut self, concept: Concept) -> Self {
        self.concept = Some(concept);
        self
    }
    pub fn limit(mut self, limit: NonZeroU32) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, Clone)]
pub struct RelationsRequest {
    pub entity: EntityId,
    pub target_type: Option<RelationEntityType>,
    pub relation_type: Option<RelationType>,
    pub limit: Option<NonZeroU32>,
}

impl RelationsRequest {
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            target_type: None,
            relation_type: None,
            limit: None,
        }
    }
    pub fn target_type(mut self, kind: RelationEntityType) -> Self {
        self.target_type = Some(kind);
        self
    }
    pub fn relation_type(mut self, kind: RelationType) -> Self {
        self.relation_type = Some(kind);
        self
    }
    pub fn limit(mut self, limit: NonZeroU32) -> Self {
        self.limit = Some(limit);
        self
    }
}
