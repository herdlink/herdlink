use crate::{
    Dataset, DiseaseId, DiseaseMapping, Error, MappingMethod, MappingOutcome, MeshId, Result,
    mondo::normalize_name,
};
use std::collections::{BTreeMap, BTreeSet};

/// Offline disease-to-HPO mapper with explicit provenance and ambiguity.
pub struct DiseaseMapper<'a> {
    dataset: &'a Dataset,
    allow_names: bool,
    overrides: BTreeMap<MeshId, DiseaseId>,
}
impl<'a> DiseaseMapper<'a> {
    pub fn new(dataset: &'a Dataset) -> Self {
        Self {
            dataset,
            allow_names: true,
            overrides: BTreeMap::new(),
        }
    }
    pub fn allow_name_fallback(mut self, allow: bool) -> Self {
        self.allow_names = allow;
        self
    }
    /// Explicit mapping for a known coverage gap; the target must have HPO annotations.
    pub fn with_override(mut self, mesh: MeshId, disease: DiseaseId) -> Result<Self> {
        if self.dataset.profile(&disease).is_none() {
            return Err(Error::Unmapped(disease.to_string()));
        }
        self.overrides.insert(mesh, disease);
        Ok(self)
    }
    fn outcome(
        &self,
        ids: impl IntoIterator<Item = DiseaseId>,
        method: MappingMethod,
        matched: &str,
    ) -> MappingOutcome {
        let ids: BTreeSet<_> = ids
            .into_iter()
            .filter_map(|id| self.dataset.profile(&id).map(|p| p.id.clone()))
            .collect();
        let mut candidates: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let p = self.dataset.profile(&id).unwrap();
                DiseaseMapping {
                    disease_id: id,
                    mondo: p.mondo.clone(),
                    annotation_ids: p.annotation_ids.clone(),
                    method,
                    matched_value: matched.into(),
                }
            })
            .collect();
        match candidates.len() {
            0 => MappingOutcome::Unmapped {
                query: matched.into(),
            },
            1 => MappingOutcome::Mapped(candidates.remove(0)),
            _ => MappingOutcome::Ambiguous(candidates),
        }
    }
    pub fn map_id(&self, id: &DiseaseId) -> MappingOutcome {
        self.outcome([id.clone()], MappingMethod::AnnotationId, id.as_str())
    }
    pub fn map_mesh(&self, mesh: &MeshId) -> MappingOutcome {
        if let Some(id) = self.overrides.get(mesh) {
            return self.outcome([id.clone()], MappingMethod::Override, mesh.as_str());
        }
        let ids = self
            .dataset
            .mondo
            .mesh_to_mondo
            .get(mesh)
            .into_iter()
            .flatten()
            .filter_map(|id| id.to_string().parse::<DiseaseId>().ok());
        self.outcome(ids, MappingMethod::ExactMesh, mesh.as_str())
    }
    pub fn map_name(&self, name: &str) -> MappingOutcome {
        self.outcome(
            self.dataset
                .names
                .get(&normalize_name(name))
                .into_iter()
                .flatten()
                .cloned(),
            MappingMethod::ExactName,
            name,
        )
    }
    pub fn map_entity(&self, entity: &pubtator3::Entity) -> Result<MappingOutcome> {
        self.map_entity_with_names(entity, &[])
    }
    /// Additional names must describe the selected entity (e.g. the user's disease query).
    pub fn map_entity_with_names(
        &self,
        entity: &pubtator3::Entity,
        names: &[&str],
    ) -> Result<MappingOutcome> {
        if entity.id.namespace() != "DISEASE" || entity.biotype != "disease" {
            return Err(Error::InvalidRequest(format!(
                "{} is not a disease entity",
                entity.id
            )));
        }
        if entity.db.as_deref() == Some("ncbi_mesh")
            && let Some(id) = &entity.db_id
        {
            let mesh: MeshId = id.parse()?;
            let mapped = self.map_mesh(&mesh);
            if !matches!(mapped, MappingOutcome::Unmapped { .. }) {
                return Ok(mapped);
            }
            // A known exact ID mapping with no annotations must not be replaced by a name guess.
            if self.dataset.mondo.mesh_to_mondo.contains_key(&mesh) {
                return Ok(mapped);
            }
        }
        if self.allow_names {
            let mut ids = BTreeSet::new();
            let mut matched = Vec::new();
            for name in std::iter::once(entity.name.as_str()).chain(names.iter().copied()) {
                if let Some(found) = self.dataset.names.get(&normalize_name(name)) {
                    ids.extend(found.iter().cloned());
                    matched.push(name);
                }
            }
            return Ok(self.outcome(ids, MappingMethod::ExactName, &matched.join(" / ")));
        }
        Ok(MappingOutcome::Unmapped {
            query: entity.id.to_string(),
        })
    }
}

impl Dataset {
    pub fn mapper(&self) -> DiseaseMapper<'_> {
        DiseaseMapper::new(self)
    }
}

impl MappingOutcome {
    pub fn require_unique(self) -> Result<DiseaseMapping> {
        match self {
            Self::Mapped(mapping) => Ok(mapping),
            Self::Ambiguous(candidates) => Err(Error::AmbiguousMapping(candidates)),
            Self::Unmapped { query } => Err(Error::Unmapped(query)),
        }
    }
}
