use crate::{
    Error, GraphBatch, Result, Store,
    model::{hash, props},
    projection,
};
use pubtator3::{
    AutocompleteRequest, Document, Entity, EntityId, ExportFormat, MeshDescriptorId, MeshSynonyms,
    Page, Pmcid, Pmid, RelatedEntity, RelationsRequest, SearchQuery, SearchResponse, TextScope,
};
use pubtator3_hpo::{
    DiseaseId, DiseaseProfile, MappedDisease, PubTatorDiseaseMatch, SimilarDisease,
    SimilarityOptions, SimilarityReport,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct CachePolicy {
    pub ttl: Duration,
}
impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(24 * 60 * 60),
        }
    }
}
#[derive(Clone)]
struct Cache {
    store: Store,
    namespace: String,
    policy: CachePolicy,
    gate: Arc<Mutex<()>>,
}
impl Cache {
    fn key(&self, operation: &str, request: &impl Serialize) -> String {
        format!(
            "cache:{}",
            hash(&("biomedical-graph-v2", &self.namespace, operation, request))
        )
    }
    async fn get<T, F>(
        &self,
        operation: &str,
        request: impl Serialize,
        fetch: F,
        project: impl FnOnce(&T) -> Result<GraphBatch>,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: Future<Output = Result<T>>,
    {
        let key = self.key(operation, &request);
        // Clones coalesce misses; independently constructed/process clients may both fetch,
        // but graph MERGEs and cache publication remain transactional and idempotent.
        let _guard = self.gate.lock().await;
        if !self.policy.ttl.is_zero()
            && let Some(hit) = self.store.cached(&key).await?
        {
            return Ok(hit);
        }
        let value = fetch.await?;
        let mut batch = project(&value)?;
        let payload = serde_json::to_string(&value)?;
        let payload_hash = self.store.write_payload(&payload).await?;
        let objects: Vec<_> = batch.nodes.keys().cloned().collect();
        let result = batch.node(format!("fetch-result:{}",hash(&(&key,&payload))),"FetchResult",props(json!({
            "operation":operation,"namespace":self.namespace,"request_json":serde_json::to_string(&request)?,"payload_hash":payload_hash,"payload_bytes":payload.len()
        })));
        for uid in objects {
            batch.edge(&result, "HAS_OBJECT", &uid, "", Default::default());
        }
        let ttl_ms = i64::try_from(self.policy.ttl.as_millis())
            .map_err(|_| Error::Invalid("cache TTL exceeds signed milliseconds".into()))?;
        batch.node(
            &key,
            "QueryCache",
            props(json!({"ttl_ms":ttl_ms,"payload_hash":payload_hash,"operation":operation})),
        );
        batch.edge(&key, "CACHED_RESULT", &result, "", Default::default());
        self.store.cache_write(&batch, &key).await?;
        Ok(value)
    }
}

#[derive(Clone)]
pub struct CachedPubTator {
    upstream: pubtator3::Client,
    cache: Cache,
}
impl CachedPubTator {
    pub fn new(upstream: pubtator3::Client, store: Store, policy: CachePolicy) -> Self {
        let namespace = upstream.cache_identity();
        Self {
            upstream,
            cache: Cache {
                store,
                namespace,
                policy,
                gate: Arc::new(Mutex::new(())),
            },
        }
    }
    pub fn store(&self) -> &Store {
        &self.cache.store
    }
    /// Useful for targeted eviction via Store::invalidate. Requests use each method's tuple layout.
    pub fn cache_key(&self, operation: &str, request: &impl Serialize) -> String {
        self.cache.key(operation, request)
    }
    pub async fn autocomplete(&self, request: &AutocompleteRequest) -> Result<Vec<Entity>> {
        self.cache
            .get(
                "autocomplete",
                (&request.query, request.concept, request.limit),
                async { Ok(self.upstream.autocomplete(request).await?) },
                |v| Ok(projection::entities(v)),
            )
            .await
    }
    pub async fn relations(&self, request: &RelationsRequest) -> Result<Vec<RelatedEntity>> {
        self.cache
            .get(
                "relations",
                (
                    &request.entity,
                    request.target_type,
                    request.relation_type,
                    request.limit,
                ),
                async { Ok(self.upstream.relations(request).await?) },
                |v| Ok(projection::relations(v, &self.cache.namespace)),
            )
            .await
    }
    pub async fn search(&self, query: &SearchQuery, page: Page) -> Result<SearchResponse> {
        self.cache
            .get(
                "search",
                (query.as_str(), page),
                async { Ok(self.upstream.search(query, page).await?) },
                |v| Ok(projection::search(v)),
            )
            .await
    }
    pub async fn annotations(&self, pmids: &[Pmid], scope: TextScope) -> Result<Vec<Document>> {
        self.cache
            .get(
                "annotations",
                (pmids, scope_name(scope)),
                async { Ok(self.upstream.annotations(pmids, scope).await?) },
                |v| {
                    Ok(projection::documents(
                        v,
                        scope_name(scope),
                        &self.cache.namespace,
                    ))
                },
            )
            .await
    }
    pub async fn annotations_batched(
        &self,
        pmids: &[Pmid],
        scope: TextScope,
    ) -> Result<Vec<Document>> {
        if pmids.is_empty() {
            return Err(Error::Invalid("provide at least one PMID".into()));
        }
        let mut documents = Vec::new();
        for batch in pmids.chunks(pubtator3::MAX_EXPORT_IDS) {
            documents.extend(self.annotations(batch, scope).await?);
        }
        Ok(documents)
    }
    pub async fn pmc_annotations(&self, pmcids: &[Pmcid]) -> Result<Vec<Document>> {
        self.cache
            .get(
                "pmc_annotations",
                pmcids,
                async { Ok(self.upstream.pmc_annotations(pmcids).await?) },
                |v| Ok(projection::documents(v, "full_text", &self.cache.namespace)),
            )
            .await
    }
    pub async fn mesh_synonyms(&self, descriptor: &MeshDescriptorId) -> Result<MeshSynonyms> {
        self.cache
            .get(
                "mesh_synonyms",
                descriptor,
                async { Ok(self.upstream.mesh_synonyms(descriptor).await?) },
                |v| Ok(projection::mesh_terms(v)),
            )
            .await
    }
    pub async fn synonyms(&self, entity: &Entity) -> Result<MeshSynonyms> {
        self.mesh_synonyms(&entity.mesh_descriptor_id()?).await
    }
    pub async fn export(
        &self,
        pmids: &[Pmid],
        format: ExportFormat,
        scope: TextScope,
    ) -> Result<String> {
        self.cache
            .get(
                "export",
                (pmids, format, scope_name(scope)),
                async { Ok(self.upstream.export(pmids, format, scope).await?) },
                |v| self.export_graph(v, format, scope_name(scope)),
            )
            .await
    }
    pub async fn pmc_export(&self, pmcids: &[Pmcid], format: ExportFormat) -> Result<String> {
        self.cache
            .get(
                "pmc_export",
                (pmcids, format),
                async { Ok(self.upstream.pmc_export(pmcids, format).await?) },
                |v| self.export_graph(v, format, "full_text"),
            )
            .await
    }
    fn export_graph(&self, value: &str, format: ExportFormat, scope: &str) -> Result<GraphBatch> {
        let mut batch = GraphBatch::default();
        batch.node(
            format!(
                "raw-export:{}",
                hash(&(&self.cache.namespace, format, scope, value))
            ),
            "RawExport",
            props(json!({"format":format,"scope":scope,"body_hash":hash(&value),"body_bytes":value.len()})),
        );
        if format == ExportFormat::BioCJson {
            batch.extend(projection::documents(
                &pubtator3::parse_documents(value)?,
                scope,
                &self.cache.namespace,
            ));
        }
        Ok(batch)
    }
}
fn scope_name(scope: TextScope) -> &'static str {
    match scope {
        TextScope::Abstract => "abstract",
        TextScope::FullText => "full_text",
    }
}

#[derive(Clone)]
pub struct CachedHpo {
    upstream: pubtator3_hpo::Client,
    cache: Cache,
}
impl CachedHpo {
    pub fn new(upstream: pubtator3_hpo::Client, store: Store, policy: CachePolicy) -> Self {
        let namespace = format!(
            "{}|{}",
            upstream.pubtator().cache_identity(),
            upstream.cache_identity()
        );
        Self {
            upstream,
            cache: Cache {
                store,
                namespace,
                policy,
                gate: Arc::new(Mutex::new(())),
            },
        }
    }
    pub fn store(&self) -> &Store {
        &self.cache.store
    }
    pub fn cache_key(&self, operation: &str, request: &impl Serialize) -> String {
        self.cache.key(operation, request)
    }
    pub async fn resolve_disease(
        &self,
        query: &str,
        selected: Option<&EntityId>,
    ) -> Result<Entity> {
        self.cache
            .get(
                "resolve_disease",
                (query, selected),
                async { Ok(self.upstream.resolve_disease(query, selected).await?) },
                |v| Ok(projection::entities(std::slice::from_ref(v))),
            )
            .await
    }
    pub async fn disease_phenotypes(
        &self,
        query: &str,
        selected: Option<&EntityId>,
    ) -> Result<MappedDisease> {
        self.cache
            .get(
                "disease_phenotypes",
                (query, selected),
                async { Ok(self.upstream.disease_phenotypes(query, selected).await?) },
                |v| {
                    Ok(projection::mapped_disease(
                        v,
                        self.upstream.dataset(),
                        &self.cache.namespace,
                    ))
                },
            )
            .await
    }
    /// Query a known disease record locally and persist its annotation/ontology subgraph.
    pub async fn profile(&self, id: &DiseaseId) -> Result<DiseaseProfile> {
        self.cache
            .get(
                "profile",
                id,
                async {
                    self.upstream
                        .dataset()
                        .profile(id)
                        .cloned()
                        .ok_or_else(|| Error::Invalid(format!("unknown HPO disease {id}")))
                },
                |v| Ok(projection::profile(v, self.upstream.dataset())),
            )
            .await
    }
    pub async fn similar_diseases(
        &self,
        query: &str,
        selected: Option<&EntityId>,
        options: &SimilarityOptions,
    ) -> Result<SimilarityReport> {
        let request = (
            query,
            selected,
            options.limit,
            options.min_score,
            options.min_phenotypes,
        );
        self.cache.get("similar_diseases",request,async { Ok(self.upstream.similar_diseases(query,selected,options).await?) },|v| {
            let mapped = MappedDisease { entity:v.entity.clone(),mapping:v.mapping.clone(),profile:v.source.clone() };
            let mut batch = projection::mapped_disease(&mapped,self.upstream.dataset(),&self.cache.namespace);
            for disease in &v.matches {
                if let Some(profile) = self.upstream.dataset().profile(&disease.disease_id) { batch.extend(projection::profile(profile,self.upstream.dataset())); }
                let uid = batch.node(format!("similarity:{}",hash(&(&self.cache.namespace,request,&v.source.id,disease))),"SimilarityResult",props(disease));
                batch.node(&uid,"SimilarityResult",props(json!({"algorithm":"simGIC","options_json":serde_json::to_string(&request)?,"corpus_diseases":v.corpus_diseases,"snapshot":self.upstream.dataset().fingerprint()})));
                batch.edge(&uid,"SOURCE",&format!("disease:{}",v.source.id),"",Default::default());
                batch.edge(&uid,"TARGET",&format!("disease:{}",disease.disease_id),"",Default::default());
            }
            Ok(batch)
        }).await
    }
    pub async fn pubtator_entities(
        &self,
        disease: &SimilarDisease,
    ) -> Result<Vec<PubTatorDiseaseMatch>> {
        self.cache
            .get(
                "pubtator_entities",
                disease,
                async { Ok(self.upstream.pubtator_entities(disease).await?) },
                |v| {
                    let mut batch = GraphBatch::default();
                    let profile = self
                        .upstream
                        .dataset()
                        .profile(&disease.disease_id)
                        .ok_or_else(|| {
                            Error::Invalid(
                                "reverse lookup disease missing from HPO snapshot".into(),
                            )
                        })?;
                    for matched in v {
                        let mapping = pubtator3_hpo::DiseaseMapping {
                            disease_id: profile.id.clone(),
                            mondo: profile.mondo.clone(),
                            annotation_ids: profile.annotation_ids.clone(),
                            method: matched.method,
                            matched_value: match matched.method {
                                pubtator3_hpo::MappingMethod::ExactName => {
                                    matched.entity.name.clone()
                                }
                                _ => matched
                                    .entity
                                    .db_id
                                    .clone()
                                    .unwrap_or_else(|| matched.entity.name.clone()),
                            },
                        };
                        batch.extend(projection::mapped_disease(
                            &MappedDisease {
                                entity: matched.entity.clone(),
                                mapping,
                                profile: profile.clone(),
                            },
                            self.upstream.dataset(),
                            &self.cache.namespace,
                        ));
                    }
                    Ok(batch)
                },
            )
            .await
    }
    pub async fn supporting_papers(
        &self,
        source: &EntityId,
        target: &EntityId,
        page: Page,
    ) -> Result<SearchResponse> {
        self.cache
            .get(
                "supporting_papers",
                (source, target, page),
                async {
                    Ok(self
                        .upstream
                        .supporting_papers(source, target, page)
                        .await?)
                },
                |v| Ok(projection::search(v)),
            )
            .await
    }
}
