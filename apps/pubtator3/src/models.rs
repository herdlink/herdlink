use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AnnotationId, DocumentId, EntityId, Error, MeshDescriptorId, MeshTermId, Page, Pmcid, Pmid,
    RelationId, RelationType, Result,
};

/// Unstructured upstream metadata is retained without assuming all infons are strings.
pub type Metadata = BTreeMap<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    #[serde(rename = "_id")]
    pub id: EntityId,
    pub name: String,
    pub biotype: String,
    #[serde(default)]
    pub db: Option<String>,
    #[serde(default)]
    pub db_id: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Upstream markup; escape or sanitize before embedding in a web page.
    #[serde(rename = "match", default)]
    pub matched_text: Option<String>,
    #[serde(flatten)]
    pub extra: Metadata,
}

impl Entity {
    /// Resolve the database ID for synonym lookup. Gene IDs and MeSH supplementary
    /// concept IDs cannot be passed to the descriptor-details endpoint.
    pub fn mesh_descriptor_id(&self) -> Result<MeshDescriptorId> {
        match (self.db.as_deref(), self.db_id.as_deref()) {
            (Some("ncbi_mesh"), Some(id)) => id.parse(),
            _ => Err(Error::UnsupportedSynonyms {
                entity: self.id.clone(),
                database: self.db.clone(),
                database_id: self.db_id.clone(),
            }),
        }
    }
}

/// MeSH descriptor terms, including its preferred label and nonpreferred entry terms.
/// Entry terms can include narrower concepts; they are not guaranteed exact synonyms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSynonyms {
    #[serde(with = "crate::ids::mesh_descriptor_uri")]
    pub descriptor: MeshDescriptorId,
    pub terms: Vec<MeshTerm>,
    #[serde(flatten)]
    pub extra: Metadata,
}

impl MeshSynonyms {
    pub fn preferred_term(&self) -> Option<&MeshTerm> {
        self.terms.iter().find(|term| term.preferred == Some(true))
    }

    /// Nonpreferred entry terms explicitly marked as such by NLM.
    pub fn synonyms(&self) -> impl Iterator<Item = &MeshTerm> {
        self.terms
            .iter()
            .filter(|term| term.preferred == Some(false))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTerm {
    #[serde(rename = "resource", with = "crate::ids::mesh_term_uri")]
    pub id: MeshTermId,
    pub label: String,
    /// The API schema permits this field to be omitted; unknown is retained as None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred: Option<bool>,
    #[serde(flatten)]
    pub extra: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source: EntityId,
    pub target: EntityId,
    pub publications: u64,
    #[serde(flatten)]
    pub extra: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<Publication>,
    pub count: u64,
    pub page_size: u32,
    pub current: Page,
    pub total_pages: u32,
    #[serde(default)]
    pub facets: Metadata,
    #[serde(flatten)]
    pub extra: Metadata,
}

impl SearchResponse {
    pub fn next_page(&self) -> Option<Page> {
        if self.current.get() < self.total_pages {
            self.current.next()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    pub pmid: Pmid,
    #[serde(default)]
    pub pmcid: Option<Pmcid>,
    pub title: String,
    #[serde(default)]
    pub journal: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    /// ISO date as returned by PubTator; bibliographic date precision varies.
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub text_hl: Option<String>,
    #[serde(default)]
    pub citations: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: Metadata,
}

/// A BioC document. The local `id` can differ from `pmid`, particularly for full text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    #[serde(default)]
    pub pmid: Option<Pmid>,
    #[serde(default)]
    pub pmcid: Option<Pmcid>,
    #[serde(default)]
    pub infons: Metadata,
    #[serde(default)]
    pub passages: Vec<Passage>,
    #[serde(default)]
    pub relations: Vec<BioCRelation>,
    #[serde(flatten)]
    pub extra: Metadata,
}

impl Document {
    /// Both passage-level and sentence-level annotations, in document order.
    pub fn annotations(&self) -> impl Iterator<Item = &Annotation> {
        self.passages.iter().flat_map(|p| {
            p.annotations
                .iter()
                .chain(p.sentences.iter().flat_map(|s| s.annotations.iter()))
        })
    }

    pub fn all_relations(&self) -> impl Iterator<Item = &BioCRelation> {
        self.relations
            .iter()
            .chain(self.passages.iter().flat_map(|p| {
                p.relations
                    .iter()
                    .chain(p.sentences.iter().flat_map(|s| s.relations.iter()))
            }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passage {
    #[serde(default)]
    pub infons: Metadata,
    /// Document-relative character offset, not a UTF-8 byte index.
    pub offset: u64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub sentences: Vec<Sentence>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub relations: Vec<BioCRelation>,
    #[serde(flatten)]
    pub extra: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    #[serde(default)]
    pub infons: Metadata,
    pub offset: u64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub relations: Vec<BioCRelation>,
    #[serde(flatten)]
    pub extra: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    #[serde(default)]
    pub infons: EntityInfo,
    pub text: String,
    pub locations: Vec<Location>,
    #[serde(flatten)]
    pub extra: Metadata,
}

/// Entity metadata shared by annotations and the role1/role2 objects in relations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityInfo {
    #[serde(rename = "type", default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub accession: Option<EntityId>,
    /// Database identifier(s), e.g. `MESH:D008070` or `4363`; distinct from accession.
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub biotype: Option<String>,
    #[serde(default)]
    pub valid: Option<bool>,
    #[serde(flatten)]
    pub extra: Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub offset: u64,
    pub length: u64,
}

impl Location {
    pub fn end(self) -> Option<u64> {
        self.offset.checked_add(self.length)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioCRelation {
    pub id: RelationId,
    #[serde(default)]
    pub infons: RelationInfo,
    #[serde(default)]
    pub nodes: Vec<RelationNode>,
    #[serde(flatten)]
    pub extra: Metadata,
}

/// BioC uses labels such as `Positive_Correlation`, rather than the query enum's
/// `positive_correlate`. Keep the original label and score representation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationInfo {
    #[serde(rename = "type", default)]
    pub relation_type: Option<String>,
    #[serde(default)]
    pub role1: Option<EntityInfo>,
    #[serde(default)]
    pub role2: Option<EntityInfo>,
    #[serde(default)]
    pub score: Option<Value>,
    #[serde(flatten)]
    pub extra: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationNode {
    /// Opaque BioC reference: PubTator's relation nodes can reference relation
    /// display entries rather than annotation IDs.
    pub refid: String,
    pub role: String,
    #[serde(flatten)]
    pub extra: Metadata,
}

/// Decode the PubTator3 wrapper, a standard BioC collection, or an array of documents.
/// Also accepts streams of JSON documents/collections (including NDJSON).
pub fn parse_documents(json: &str) -> Result<Vec<Document>> {
    let mut documents = Vec::new();
    for item in serde_json::Deserializer::from_str(json).into_iter::<Value>() {
        match item? {
            Value::Object(mut object) => {
                if let Some(collection) = object
                    .remove("PubTator3")
                    .or_else(|| object.remove("documents"))
                {
                    documents.extend(serde_json::from_value::<Vec<Document>>(collection)?);
                } else {
                    documents.push(serde_json::from_value(Value::Object(object))?);
                }
            }
            array @ Value::Array(_) => {
                documents.extend(serde_json::from_value::<Vec<Document>>(array)?)
            }
            other => documents.push(serde_json::from_value(other)?),
        }
    }
    if json.trim().is_empty() {
        // An empty HTTP body is not an empty BioC collection.
        let _: Value = serde_json::from_str(json)?;
    }
    Ok(documents)
}
