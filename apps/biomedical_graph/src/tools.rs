//! Function declarations and dispatch for the cached clients. No model is needed
//! to call [`GraphTools::execute`]. Tools preserve the clients' full JSON results.
use crate::{CachedHpo, CachedPubTator, Error, Result};
use openai::{FunctionCall, InputItem, Tool};
use pubtator3::{
    AutocompleteRequest, Concept, Entity, EntityId, ExportFormat, MeshDescriptorId, Page, Pmcid,
    Pmid, RelationEntityType, RelationType, RelationsRequest, SearchQuery, TextScope,
};
use pubtator3_hpo::{DiseaseId, SimilarDisease, SimilarityOptions};
use serde::Deserialize;
use serde_json::{Value, json};
use std::num::{NonZeroU32, NonZeroUsize};

fn object(fields: &[(&str, Value)]) -> Value {
    json!({
        "type": "object",
        "properties": fields.iter().map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect::<serde_json::Map<_, _>>(),
        "required": fields.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
        "additionalProperties": false
    })
}
fn text(description: &str) -> Value {
    json!({"type": "string", "description": description})
}
fn choice(values: &[&str]) -> Value {
    json!({"type": "string", "enum": values})
}
fn nullable(mut schema: Value) -> Value {
    let kind = schema["type"].take();
    schema["type"] = json!([kind, "null"]);
    if let Some(values) = schema.get_mut("enum") {
        values.as_array_mut().unwrap().push(Value::Null);
    }
    schema
}
fn ids(kind: &str, max: Option<usize>) -> Value {
    let mut schema = json!({"type": "array", "items": text(kind), "minItems": 1});
    if let Some(max) = max {
        schema["maxItems"] = json!(max);
    }
    schema
}
fn function(name: &str, description: &str, fields: &[(&str, Value)]) -> Tool {
    Tool::function(name, description, object(fields))
}
fn scope() -> Value {
    choice(&["abstract", "full_text"])
}
fn format() -> Value {
    choice(&["pubtator", "biocxml", "biocjson"])
}
fn limit() -> Value {
    json!({"type": ["integer", "null"], "minimum": 1, "maximum": u32::MAX,
        "description": "Maximum results; null uses the upstream default."})
}
fn disease_query() -> Vec<(&'static str, Value)> {
    vec![
        ("query", text("Disease name to resolve.")),
        (
            "selected",
            nullable(text(
                "Exact PubTator @DISEASE_ accession, or null for automatic resolution.",
            )),
        ),
    ]
}

/// Ten tools matching the public CachedPubTator data methods.
pub fn pubtator_tools() -> Vec<Tool> {
    let pmids = || {
        ids(
            "Positive decimal PMID, e.g. 19894120.",
            Some(pubtator3::MAX_EXPORT_IDS),
        )
    };
    let pmcids = || {
        ids(
            "PMC accession, e.g. PMC6142073.",
            Some(pubtator3::MAX_EXPORT_IDS),
        )
    };
    vec![
        function(
            "pubtator_autocomplete",
            "Resolve text to PubTator entities using the persistent cache.",
            &[
                ("query", text("Entity name or partial name.")),
                (
                    "concept",
                    nullable(choice(&[
                        "gene", "disease", "chemical", "variant", "species", "cellline",
                    ])),
                ),
                ("limit", limit()),
            ],
        ),
        function(
            "pubtator_relations",
            "Discover ordered entity relations and supporting-publication counts; counts are not confidence scores.",
            &[
                (
                    "entity",
                    text("PubTator @ accession, e.g. @DISEASE_Huntington_Disease."),
                ),
                (
                    "target_type",
                    nullable(choice(&["gene", "disease", "chemical", "variant"])),
                ),
                (
                    "relation_type",
                    nullable(choice(&[
                        "treat",
                        "cause",
                        "cotreat",
                        "convert",
                        "compare",
                        "interact",
                        "associate",
                        "positive_correlate",
                        "negative_correlate",
                        "prevent",
                        "inhibit",
                        "stimulate",
                        "drug_interact",
                    ])),
                ),
                ("limit", limit()),
            ],
        ),
        function(
            "pubtator_search",
            "Search publications with free text or PubTator query syntax. Returns one complete page.",
            &[
                (
                    "query",
                    text("Search expression, including @ accessions or relations: queries."),
                ),
                (
                    "page",
                    json!({"type":"integer", "minimum":1, "maximum":u32::MAX}),
                ),
            ],
        ),
        function(
            "pubtator_annotations",
            "Get structured BioC annotations for up to 100 PMIDs. Full text is cached on disk; structured annotations are projected into Neo4j.",
            &[("pmids", pmids()), ("scope", scope())],
        ),
        function(
            "pubtator_annotations_batched",
            "Get BioC annotations for any nonempty PMID list, split into cached batches of 100.",
            &[
                ("pmids", ids("Positive decimal PMID.", None)),
                ("scope", scope()),
            ],
        ),
        function(
            "pubtator_pmc_annotations",
            "Get full-text BioC annotations for up to 100 PMC accessions.",
            &[("pmcids", pmcids())],
        ),
        function(
            "pubtator_mesh_synonyms",
            "List preferred and nonpreferred MeSH entry terms for a descriptor. Entry terms are not necessarily exact synonyms.",
            &[(
                "descriptor",
                text("MeSH descriptor ID, e.g. D000690; not an @ accession or supplementary C ID."),
            )],
        ),
        function(
            "pubtator_synonyms",
            "List MeSH entry terms for an entity from autocomplete. Requires db=ncbi_mesh and a descriptor db_id.",
            &[(
                "entity",
                object(&[
                    ("_id", text("PubTator @ accession.")),
                    ("name", text("Entity name.")),
                    ("biotype", text("Entity biotype.")),
                    ("db", nullable(text("Entity database."))),
                    ("db_id", nullable(text("Entity database ID."))),
                ]),
            )],
        ),
        function(
            "pubtator_export",
            "Get a raw PubTator, BioC XML or BioC JSON export for up to 100 PMIDs. Full text requires biocxml or biocjson. Returns the entire body as a JSON string.",
            &[("pmids", pmids()), ("format", format()), ("scope", scope())],
        ),
        function(
            "pubtator_pmc_export",
            "Get a raw full-text export for up to 100 PMC accessions. Returns the entire body as a JSON string.",
            &[
                ("pmcids", pmcids()),
                ("format", choice(&["biocxml", "biocjson"])),
            ],
        ),
    ]
}

/// Six tools matching the public CachedHpo data methods. Requires a local dataset.
pub fn hpo_tools() -> Vec<Tool> {
    let phenotype = || {
        object(&[
            ("id", text("HP identifier.")),
            ("name", text("Phenotype name.")),
        ])
    };
    let phenotypes = || json!({"type":"array", "items":phenotype()});
    let mut similarity = disease_query();
    similarity.extend([
        ("limit", json!({"type":"integer", "minimum":1})),
        (
            "min_score",
            json!({"type":"number", "minimum":0, "maximum":1}),
        ),
        ("min_phenotypes", json!({"type":"integer", "minimum":0})),
    ]);
    vec![
        function(
            "hpo_resolve_disease",
            "Resolve a disease name to a PubTator entity; ambiguous names may require selected.",
            &disease_query(),
        ),
        function(
            "hpo_disease_phenotypes",
            "Map a PubTator disease through Mondo to the loaded HPO profile, preserving positive, absent and conflicting annotations.",
            &disease_query(),
        ),
        function(
            "hpo_profile",
            "Read and persist a known disease profile from the local HPO dataset, without HTTP.",
            &[(
                "id",
                text("MONDO, OMIM, ORPHA or DECIPHER disease ID; not an HP feature ID."),
            )],
        ),
        function(
            "hpo_similar_diseases",
            "Rank the loaded HPO disease corpus by simGIC phenotype similarity. Scores are not probabilities or biological relations.",
            &similarity,
        ),
        function(
            "hpo_pubtator_entities",
            "Map a SimilarDisease object from hpo_similar_diseases back to PubTator. Copy the complete match object into disease.",
            &[(
                "disease",
                object(&[
                    ("disease_id", text("Curated disease ID.")),
                    ("name", text("Disease name.")),
                    ("mondo", nullable(text("Mondo ID."))),
                    (
                        "annotation_ids",
                        json!({"type":"array", "items":text("Disease annotation record ID.")}),
                    ),
                    (
                        "mesh_ids",
                        json!({"type":"array", "items":text("MeSH ID.")}),
                    ),
                    ("score", json!({"type":"number", "minimum":0, "maximum":1})),
                    ("phenotype_count", json!({"type":"integer", "minimum":0})),
                    ("shared_phenotypes", phenotypes()),
                    ("source_only_phenotypes", phenotypes()),
                    ("conflicting_phenotypes", phenotypes()),
                ]),
            )],
        ),
        function(
            "hpo_supporting_papers",
            "Search one page of papers co-mentioning two PubTator disease entities. Co-mentions do not establish causation.",
            &[
                ("source", text("Source PubTator @DISEASE_ accession.")),
                ("target", text("Target PubTator @DISEASE_ accession.")),
                (
                    "page",
                    json!({"type":"integer", "minimum":1, "maximum":u32::MAX}),
                ),
            ],
        ),
    ]
}

#[derive(Clone)]
pub struct GraphTools {
    pubtator: CachedPubTator,
    hpo: Option<CachedHpo>,
}
impl GraphTools {
    pub fn new(pubtator: CachedPubTator) -> Self {
        Self {
            pubtator,
            hpo: None,
        }
    }
    pub fn with_hpo(mut self, hpo: CachedHpo) -> Self {
        self.hpo = Some(hpo);
        self
    }
    /// Register only tools whose clients have been configured.
    pub fn definitions(&self) -> Vec<Tool> {
        let mut tools = pubtator_tools();
        if self.hpo.is_some() {
            tools.extend(hpo_tools());
        }
        tools
    }
    fn hpo(&self) -> Result<&CachedHpo> {
        self.hpo.as_ref().ok_or_else(|| {
            Error::Invalid("HPO tools require a configured CachedHpo dataset".into())
        })
    }
    /// Dispatch a completed call. Invalid arguments/unknown tools never become cache entries.
    pub async fn execute(&self, name: &str, arguments: &str) -> Result<Value> {
        macro_rules! args {
            ($ty:ty) => {
                serde_json::from_str::<$ty>(arguments)?
            };
        }
        macro_rules! output {
            ($call:expr) => {
                Ok(serde_json::to_value($call.await?)?)
            };
        }
        match name {
            "pubtator_autocomplete" => {
                let a = args!(AutocompleteArgs);
                if a.query.trim().is_empty() {
                    return Err(Error::Invalid("query cannot be empty".into()));
                }
                output!(self.pubtator.autocomplete(&AutocompleteRequest {
                    query: a.query,
                    concept: a.concept,
                    limit: a.limit
                }))
            }
            "pubtator_relations" => {
                let a = args!(RelationsArgs);
                output!(self.pubtator.relations(&RelationsRequest {
                    entity: a.entity,
                    target_type: a.target_type,
                    relation_type: a.relation_type,
                    limit: a.limit
                }))
            }
            "pubtator_search" => {
                let a = args!(SearchArgs);
                output!(self.pubtator.search(&SearchQuery::text(a.query)?, a.page))
            }
            "pubtator_annotations" | "pubtator_annotations_batched" => {
                let a = args!(AnnotationsArgs);
                if name == "pubtator_annotations" {
                    output!(self.pubtator.annotations(&a.pmids, a.scope.into()))
                } else {
                    output!(self.pubtator.annotations_batched(&a.pmids, a.scope.into()))
                }
            }
            "pubtator_pmc_annotations" => {
                let a = args!(PmcArgs);
                output!(self.pubtator.pmc_annotations(&a.pmcids))
            }
            "pubtator_mesh_synonyms" => {
                let a = args!(MeshArgs);
                output!(self.pubtator.mesh_synonyms(&a.descriptor))
            }
            "pubtator_synonyms" => {
                let a = args!(SynonymsArgs);
                output!(self.pubtator.synonyms(&Entity {
                    id: a.entity.id,
                    name: a.entity.name,
                    biotype: a.entity.biotype,
                    db: a.entity.db,
                    db_id: a.entity.db_id,
                    description: None,
                    matched_text: None,
                    extra: Default::default(),
                }))
            }
            "pubtator_export" => {
                let a = args!(ExportArgs);
                output!(self.pubtator.export(&a.pmids, a.format, a.scope.into()))
            }
            "pubtator_pmc_export" => {
                let a = args!(PmcExportArgs);
                output!(self.pubtator.pmc_export(&a.pmcids, a.format))
            }
            "hpo_resolve_disease" | "hpo_disease_phenotypes" => {
                let a = args!(DiseaseArgs);
                if name == "hpo_resolve_disease" {
                    output!(self.hpo()?.resolve_disease(&a.query, a.selected.as_ref()))
                } else {
                    output!(
                        self.hpo()?
                            .disease_phenotypes(&a.query, a.selected.as_ref())
                    )
                }
            }
            "hpo_profile" => {
                let a = args!(ProfileArgs);
                output!(self.hpo()?.profile(&a.id))
            }
            "hpo_similar_diseases" => {
                let a = args!(SimilarityArgs);
                if !(0.0..=1.0).contains(&a.min_score) {
                    return Err(Error::Invalid("min_score must be in 0..=1".into()));
                }
                output!(self.hpo()?.similar_diseases(
                    &a.query,
                    a.selected.as_ref(),
                    &SimilarityOptions {
                        limit: a.limit,
                        min_score: a.min_score,
                        min_phenotypes: a.min_phenotypes,
                    }
                ))
            }
            "hpo_pubtator_entities" => {
                let a = args!(ReverseArgs);
                output!(self.hpo()?.pubtator_entities(&a.disease))
            }
            "hpo_supporting_papers" => {
                let a = args!(SupportingArgs);
                output!(self.hpo()?.supporting_papers(&a.source, &a.target, a.page))
            }
            _ => Err(Error::Invalid(format!("unknown biomedical tool: {name}"))),
        }
    }
    /// Convert success or failure to the matching Responses API tool output.
    pub async fn call(&self, call: &FunctionCall) -> InputItem {
        let result = match self.execute(&call.name, &call.arguments).await {
            Ok(value) => value,
            Err(error) => json!({"error": error.to_string()}),
        };
        InputItem::tool_output(&call.call_id, result.to_string())
    }
}

// Nullable fields can be omitted by direct Rust/JSON callers; strict model schemas
// require all properties and use null for optional filters.
macro_rules! arguments {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct $name { $($field: $ty),* }
    };
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Scope {
    Abstract,
    FullText,
}
impl From<Scope> for TextScope {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Abstract => Self::Abstract,
            Scope::FullText => Self::FullText,
        }
    }
}
arguments!(AutocompleteArgs { query: String, concept: Option<Concept>, limit: Option<NonZeroU32> });
arguments!(RelationsArgs { entity: EntityId, target_type: Option<RelationEntityType>, relation_type: Option<RelationType>, limit: Option<NonZeroU32> });
arguments!(SearchArgs {
    query: String,
    page: Page
});
arguments!(AnnotationsArgs { pmids: Vec<Pmid>, scope: Scope });
arguments!(PmcArgs { pmcids: Vec<Pmcid> });
arguments!(MeshArgs {
    descriptor: MeshDescriptorId
});
arguments!(SynonymsArgs {
    entity: SynonymEntity
});
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SynonymEntity {
    #[serde(rename = "_id")]
    id: EntityId,
    name: String,
    biotype: String,
    db: Option<String>,
    db_id: Option<String>,
}
arguments!(ExportArgs { pmids: Vec<Pmid>, format: ExportFormat, scope: Scope });
arguments!(PmcExportArgs { pmcids: Vec<Pmcid>, format: ExportFormat });
arguments!(DiseaseArgs { query: String, selected: Option<EntityId> });
arguments!(ProfileArgs { id: DiseaseId });
arguments!(SimilarityArgs { query: String, selected: Option<EntityId>, limit: NonZeroUsize, min_score: f64, min_phenotypes: usize });
arguments!(ReverseArgs {
    disease: SimilarDisease
});
arguments!(SupportingArgs {
    source: EntityId,
    target: EntityId,
    page: Page
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn check_strict_objects(schema: &Value) {
        if schema["type"] == "object" {
            assert_eq!(schema["additionalProperties"], false);
            let properties = schema["properties"].as_object().unwrap();
            let required = schema["required"].as_array().unwrap();
            assert_eq!(properties.len(), required.len());
            for name in properties.keys() {
                assert!(required.contains(&json!(name)));
            }
        }
        if let Some(properties) = schema["properties"].as_object() {
            for child in properties.values() {
                check_strict_objects(child);
            }
        }
        if let Some(items) = schema.get("items") {
            check_strict_objects(items);
        }
    }

    #[test]
    fn declarations_are_unique_and_nested_objects_are_strict() {
        let mut definitions = pubtator_tools();
        assert_eq!(definitions.len(), 10);
        let hpo = hpo_tools();
        assert_eq!(hpo.len(), 6);
        definitions.extend(hpo);
        let mut names = BTreeSet::new();
        for tool in definitions {
            let Tool::Function {
                name,
                strict,
                parameters,
                ..
            } = tool;
            assert!(names.insert(name));
            assert!(strict);
            check_strict_objects(&parameters);
        }
    }

    #[test]
    fn arguments_enforce_ids_enums_positive_pages_and_no_extra_fields() {
        assert!(serde_json::from_value::<SearchArgs>(json!({"query":"HTT", "page":0})).is_err());
        assert!(
            serde_json::from_value::<AnnotationsArgs>(
                json!({"pmids":["PMC1"], "scope":"abstract"})
            )
            .is_err()
        );
        assert!(serde_json::from_value::<PmcArgs>(json!({"pmcids":["19894120"]})).is_err());
        assert!(
            serde_json::from_value::<MeshArgs>(json!({"descriptor":"@DISEASE_Huntington_Disease"}))
                .is_err()
        );
        assert!(
            serde_json::from_value::<AutocompleteArgs>(
                json!({"query":"HTT", "concept":"gene", "limit":0})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<AnnotationsArgs>(json!({"pmids":["1"], "scope":"all"}))
                .is_err()
        );
        assert!(
            serde_json::from_value::<SearchArgs>(
                json!({"query":"HTT", "page":1, "cypher":"MATCH (n) RETURN n"})
            )
            .is_err()
        );
        assert!(serde_json::from_value::<ProfileArgs>(json!({"id":"HP:0002072"})).is_err());
        let args: AnnotationsArgs =
            serde_json::from_value(json!({"pmids":["19894120", 29355051], "scope":"full_text"}))
                .unwrap();
        assert_eq!(args.pmids.len(), 2);
        assert!(matches!(TextScope::from(args.scope), TextScope::FullText));
    }
}
