use crate::{
    Dataset, DiseaseId, DiseaseMapper, DiseaseMapping, DiseaseProfile, Error, MappingMethod,
    MappingOutcome, MeshId, Result, SimilarDisease, SimilarityOptions, SimilarityReport,
    mondo::normalize_name,
};
use pubtator3::{
    AutocompleteRequest, Concept, Entity, EntityId, Page, SearchQuery, SearchResponse,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
pub struct Client {
    pubtator: pubtator3::Client,
    dataset: Arc<Dataset>,
    allow_names: bool,
    overrides: BTreeMap<MeshId, DiseaseId>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappedDisease {
    pub entity: Entity,
    pub mapping: DiseaseMapping,
    pub profile: DiseaseProfile,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PubTatorDiseaseMatch {
    pub entity: Entity,
    pub method: MappingMethod,
}

impl Client {
    pub fn new(pubtator: pubtator3::Client, dataset: impl Into<Arc<Dataset>>) -> Self {
        Self {
            pubtator,
            dataset: dataset.into(),
            allow_names: true,
            overrides: BTreeMap::new(),
        }
    }
    pub fn allow_name_fallback(mut self, allow: bool) -> Self {
        self.allow_names = allow;
        self
    }
    pub fn with_override(mut self, mesh: MeshId, disease: DiseaseId) -> Result<Self> {
        if self.dataset.profile(&disease).is_none() {
            return Err(Error::Unmapped(disease.to_string()));
        }
        self.overrides.insert(mesh, disease);
        Ok(self)
    }
    pub fn pubtator(&self) -> &pubtator3::Client {
        &self.pubtator
    }
    pub fn dataset(&self) -> &Dataset {
        &self.dataset
    }
    pub fn mapper(&self) -> DiseaseMapper<'_> {
        let mut mapper = self.dataset.mapper().allow_name_fallback(self.allow_names);
        for (mesh, disease) in &self.overrides {
            mapper = mapper
                .with_override(mesh.clone(), disease.clone())
                .expect("validated override");
        }
        mapper
    }

    pub async fn resolve_disease(
        &self,
        query: &str,
        selected: Option<&EntityId>,
    ) -> Result<Entity> {
        let candidates = self
            .pubtator
            .autocomplete(
                &AutocompleteRequest::new(query)
                    .concept(Concept::Disease)
                    .limit(std::num::NonZeroU32::new(100).unwrap()),
            )
            .await?;
        if let Some(selected) = selected {
            return candidates
                .into_iter()
                .find(|e| &e.id == selected)
                .ok_or_else(|| {
                    Error::NoDisease(format!("{selected} was not returned for {query:?}"))
                });
        }
        let exact: Vec<_> = candidates
            .iter()
            .filter(|e| normalize_name(&e.name) == normalize_name(query))
            .collect();
        if exact.len() == 1 {
            return Ok(exact[0].clone());
        }
        if candidates.len() == 1 {
            return Ok(candidates.into_iter().next().unwrap());
        }
        if candidates.is_empty() {
            return Err(Error::NoDisease(query.into()));
        }
        Err(Error::AmbiguousDisease {
            query: query.into(),
            candidates: candidates.into_iter().map(|e| e.id).collect(),
        })
    }

    pub async fn disease_phenotypes(
        &self,
        query: &str,
        selected: Option<&EntityId>,
    ) -> Result<MappedDisease> {
        let entity = self.resolve_disease(query, selected).await?;
        let mapping = self
            .mapper()
            .map_entity_with_names(&entity, &[query])?
            .require_unique()?;
        let profile = self
            .dataset
            .profile(&mapping.disease_id)
            .expect("mapped profile")
            .clone();
        Ok(MappedDisease {
            entity,
            mapping,
            profile,
        })
    }

    /// Resolve via PubTator, map via Mondo/HPO, then rank the full local phenotype corpus.
    /// CPU work runs off the async executor; NCBI is not called for every candidate.
    pub async fn similar_diseases(
        &self,
        query: &str,
        selected: Option<&EntityId>,
        options: &SimilarityOptions,
    ) -> Result<SimilarityReport> {
        let mapped = self.disease_phenotypes(query, selected).await?;
        let dataset = self.dataset.clone();
        let options = options.clone();
        let id = mapped.mapping.disease_id.clone();
        let matches = tokio::task::spawn_blocking(move || dataset.similar(&id, &options)).await??;
        Ok(SimilarityReport {
            entity: mapped.entity,
            mapping: mapped.mapping,
            source: mapped.profile,
            matches,
            corpus_diseases: self.dataset.disease_count(),
        })
    }

    /// Map a ranked HPO disease back to PubTator using its label and up to four exact
    /// Mondo synonyms. Stops at unique ID matches. Coverage is not exhaustive.
    pub async fn pubtator_entities(
        &self,
        disease: &SimilarDisease,
    ) -> Result<Vec<PubTatorDiseaseMatch>> {
        let mut names = vec![disease.name.clone()];
        if let Some(term) = disease
            .mondo
            .as_ref()
            .and_then(|id| self.dataset.mondo.terms.get(id))
        {
            let mut synonyms: Vec<_> = term
                .names
                .iter()
                .filter(|name| normalize_name(name) != normalize_name(&disease.name))
                .cloned()
                .collect();
            synonyms.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
            let mut seen = std::collections::BTreeSet::new();
            synonyms.retain(|name| seen.insert(normalize_name(name)));
            names.extend(synonyms.into_iter().take(4));
        }
        let mut matches = BTreeMap::new();
        for name in &names {
            let candidates = self
                .pubtator
                .autocomplete(
                    &AutocompleteRequest::new(name)
                        .concept(Concept::Disease)
                        .limit(std::num::NonZeroU32::new(100).unwrap()),
                )
                .await?;
            for entity in candidates {
                if entity.id.namespace() != "DISEASE" || entity.biotype != "disease" {
                    continue;
                }
                let mesh = (entity.db.as_deref() == Some("ncbi_mesh"))
                    .then_some(entity.db_id.as_deref())
                    .flatten()
                    .and_then(|id| id.parse::<MeshId>().ok());
                let id_mapping = mesh.as_ref().map(|id| self.mapper().map_mesh(id));
                let conflicting_id = mesh.as_ref().is_some_and(|id| {
                    self.dataset.mondo.mesh_to_mondo.contains_key(id)
                        || self.overrides.contains_key(id)
                });
                let method = if let Some(MappingOutcome::Mapped(mapping)) = &id_mapping
                    && mapping.disease_id == disease.disease_id
                {
                    Some(mapping.method)
                } else if self.allow_names
                    && !conflicting_id
                    && self
                        .dataset
                        .names
                        .get(&normalize_name(&entity.name))
                        .is_some_and(|ids| ids.len() == 1 && ids.contains(&disease.disease_id))
                {
                    Some(MappingMethod::ExactName)
                } else {
                    None
                };
                if let Some(method) = method {
                    matches.insert(entity.id.clone(), PubTatorDiseaseMatch { entity, method });
                }
            }
            if matches
                .values()
                .any(|m| matches!(m.method, MappingMethod::ExactMesh | MappingMethod::Override))
            {
                return Ok(matches
                    .into_values()
                    .filter(|m| {
                        matches!(m.method, MappingMethod::ExactMesh | MappingMethod::Override)
                    })
                    .collect());
            }
        }
        Ok(matches.into_values().collect())
    }

    /// Papers mentioning both selected diseases. This is co-mention, not a typed relation.
    /// Call `pubtator_entities` first to inspect any ambiguous reverse mapping.
    pub async fn supporting_papers(
        &self,
        source: &EntityId,
        target: &EntityId,
        page: Page,
    ) -> Result<SearchResponse> {
        if source.namespace() != "DISEASE" || target.namespace() != "DISEASE" {
            return Err(Error::InvalidRequest(
                "supporting_papers requires two disease entities".into(),
            ));
        }
        Ok(self
            .pubtator
            .search(
                &SearchQuery::entity(source).and(SearchQuery::entity(target)),
                page,
            )
            .await?)
    }
}
