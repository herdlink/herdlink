use crate::{DiseaseId, HpoId, MeshId, MondoId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Original OBO metadata. Raw tag values preserve definitions, synonym scopes,
/// cross-references and future OBO fields without interpreting them as equivalences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyTerm {
    pub id: HpoId,
    pub name: String,
    pub parents: Vec<HpoId>,
    pub alternative_ids: Vec<HpoId>,
    pub obsolete: bool,
    pub replacements: Vec<HpoId>,
    pub tags: std::collections::BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhenotypeAnnotation {
    pub disease_id: DiseaseId,
    pub phenotype: HpoId,
    pub excluded: bool,
    pub reference: String,
    pub evidence: String,
    pub onset: Option<String>,
    pub frequency: Option<String>,
    pub sex: Option<String>,
    pub modifier: Option<String>,
    pub aspect: String,
    pub biocuration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phenotype {
    pub id: HpoId,
    pub name: String,
}

/// Union of annotations only when disease records have a unique exact Mondo mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseProfile {
    pub id: DiseaseId,
    pub name: String,
    pub mondo: Option<MondoId>,
    pub annotation_ids: Vec<DiseaseId>,
    pub mesh_ids: Vec<MeshId>,
    pub phenotypes: Vec<Phenotype>,
    pub excluded_phenotypes: Vec<Phenotype>,
    /// The same feature is reported both present and absent across records.
    pub conflicting_phenotypes: Vec<Phenotype>,
    pub annotations: Vec<PhenotypeAnnotation>,
    /// Missing, obsolete, or non-phenotypic HPO references omitted from scoring.
    pub unscored_annotations: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingMethod {
    ExactMesh,
    /// Exact normalized label or EXACT synonym; lexical evidence, not an ID mapping.
    ExactName,
    AnnotationId,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseMapping {
    pub disease_id: DiseaseId,
    pub mondo: Option<MondoId>,
    pub annotation_ids: Vec<DiseaseId>,
    pub method: MappingMethod,
    pub matched_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MappingOutcome {
    Mapped(DiseaseMapping),
    Ambiguous(Vec<DiseaseMapping>),
    Unmapped { query: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarDisease {
    pub disease_id: DiseaseId,
    pub name: String,
    pub mondo: Option<MondoId>,
    pub annotation_ids: Vec<DiseaseId>,
    pub mesh_ids: Vec<MeshId>,
    /// Information-content-weighted ancestor overlap (0..1), not a probability.
    pub score: f64,
    pub phenotype_count: usize,
    /// Exact shared positive terms; the semantic score can also match related terms.
    pub shared_phenotypes: Vec<Phenotype>,
    pub source_only_phenotypes: Vec<Phenotype>,
    /// Direct positive/negative contradictions in either direction. No score penalty.
    pub conflicting_phenotypes: Vec<Phenotype>,
}

#[derive(Debug, Clone)]
pub struct SimilarityOptions {
    pub limit: std::num::NonZeroUsize,
    pub min_score: f64,
    pub min_phenotypes: usize,
}
impl Default for SimilarityOptions {
    fn default() -> Self {
        Self {
            limit: std::num::NonZeroUsize::new(10).unwrap(),
            min_score: 0.0,
            min_phenotypes: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityReport {
    pub entity: pubtator3::Entity,
    pub mapping: DiseaseMapping,
    pub source: DiseaseProfile,
    pub matches: Vec<SimilarDisease>,
    pub corpus_diseases: usize,
}

pub(crate) fn term_ids(profile: &DiseaseProfile) -> BTreeSet<HpoId> {
    profile.phenotypes.iter().map(|p| p.id.clone()).collect()
}
