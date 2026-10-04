use crate::{
    DiseaseId, DiseaseProfile, Error, HpoId, Phenotype, Result, SimilarDisease, SimilarityOptions,
    annotations,
    models::term_ids,
    mondo::{self, MondoIndex},
    ontology,
};
use hpo::annotations::AnnotationId;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

/// Immutable local HPO/Mondo snapshot. Loading and ranking make no HTTP requests.
pub struct Dataset {
    pub(crate) mondo: MondoIndex,
    pub(crate) profiles: BTreeMap<DiseaseId, DiseaseProfile>,
    pub(crate) aliases: BTreeMap<DiseaseId, DiseaseId>,
    pub(crate) names: BTreeMap<String, BTreeSet<DiseaseId>>,
    closures: BTreeMap<DiseaseId, BTreeSet<HpoId>>,
    weights: BTreeMap<HpoId, f64>,
    ontology: ontology::ParsedOntology,
    pub hpo_version: String,
    pub skipped_annotation_namespaces: usize,
    fingerprint: String,
}

impl Dataset {
    /// Read `hp.obo`, `phenotype.hpoa`, and `mondo.json` from a local directory.
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        Self::from_readers(
            BufReader::new(File::open(path.join("hp.obo"))?),
            BufReader::new(File::open(path.join("phenotype.hpoa"))?),
            BufReader::new(File::open(path.join("mondo.json"))?),
        )
    }

    /// Useful for version-pinned in-memory snapshots and test fixtures.
    pub fn from_readers(
        mut hp: impl BufRead,
        mut annotations: impl Read,
        mut mondo: impl Read,
    ) -> Result<Self> {
        use sha2::{Digest, Sha256};
        // Hash the actual inputs, including manual snapshots with no manifest.
        let mut digest = Sha256::new();
        let mut bytes = Vec::new();
        hp.read_to_end(&mut bytes)?;
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(&bytes);
        let parsed = ontology::read(std::io::Cursor::new(&bytes))?;
        bytes.clear();
        annotations.read_to_end(&mut bytes)?;
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(&bytes);
        let records = annotations::read(bytes.as_slice())?;
        bytes.clear();
        mondo.read_to_end(&mut bytes)?;
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(&bytes);
        let mondo = MondoIndex::read(bytes.as_slice())?;
        drop(bytes);
        let fingerprint = format!("{:x}", digest.finalize());
        let mut profiles = BTreeMap::<DiseaseId, DiseaseProfile>::new();
        let mut aliases = BTreeMap::new();
        let mut names = BTreeMap::<String, BTreeSet<DiseaseId>>::new();
        for (record_id, record) in records.diseases {
            let canonical = mondo
                .disease_to_mondo
                .get(&record_id)
                .filter(|ids| ids.len() == 1)
                .and_then(|ids| ids.first())
                .cloned();
            let id = canonical
                .as_ref()
                .map(|id| id.to_string().parse())
                .transpose()?
                .unwrap_or_else(|| record_id.clone());
            aliases.insert(record_id.clone(), id.clone());
            names
                .entry(mondo::normalize_name(&record.name))
                .or_default()
                .insert(id.clone());
            let profile = profiles.entry(id.clone()).or_insert_with(|| {
                let term = canonical.as_ref().and_then(|id| mondo.terms.get(id));
                DiseaseProfile {
                    id,
                    name: term.map_or_else(|| record.name.clone(), |t| t.name.clone()),
                    mondo: canonical,
                    annotation_ids: Vec::new(),
                    mesh_ids: term
                        .map(|t| t.mesh.iter().cloned().collect())
                        .unwrap_or_default(),
                    phenotypes: Vec::new(),
                    excluded_phenotypes: Vec::new(),
                    conflicting_phenotypes: Vec::new(),
                    annotations: Vec::new(),
                    unscored_annotations: 0,
                }
            });
            profile.annotation_ids.push(record_id);
            profile.annotations.extend(record.annotations);
        }
        // Index exact synonyms only. A related synonym is not an automatic mapping.
        for (mondo_id, term) in &mondo.terms {
            let id: DiseaseId = mondo_id.to_string().parse()?;
            if profiles.contains_key(&id) {
                aliases.insert(id.clone(), id.clone());
                for name in &term.names {
                    names
                        .entry(mondo::normalize_name(name))
                        .or_default()
                        .insert(id.clone());
                }
            }
        }
        let mut closures = BTreeMap::new();
        let mut counts = BTreeMap::<HpoId, usize>::new();
        for (id, profile) in &mut profiles {
            let mut positive = BTreeSet::new();
            let mut negative = BTreeSet::new();
            for annotation in &mut profile.annotations {
                if annotation.aspect != "P" {
                    profile.unscored_annotations += 1;
                    continue;
                }
                let canonical = parsed
                    .aliases
                    .get(&annotation.phenotype)
                    .unwrap_or(&annotation.phenotype)
                    .clone();
                let Some(term) = parsed.ontology.hpo(canonical.number()) else {
                    profile.unscored_annotations += 1;
                    continue;
                };
                if canonical.number() == 118 || !term.all_parent_ids().contains(&118u32.into()) {
                    profile.unscored_annotations += 1;
                    continue;
                }
                annotation.phenotype = canonical.clone();
                if annotation.excluded {
                    negative.insert(canonical);
                } else {
                    positive.insert(canonical);
                }
            }
            let feature = |id: &HpoId| Phenotype {
                id: id.clone(),
                name: parsed
                    .ontology
                    .hpo(id.number())
                    .expect("validated phenotype")
                    .name()
                    .into(),
            };
            profile.phenotypes = positive.iter().map(feature).collect();
            profile.excluded_phenotypes = negative.iter().map(feature).collect();
            profile.conflicting_phenotypes =
                positive.intersection(&negative).map(feature).collect();
            let mut closure = positive;
            for phenotype in &profile.phenotypes {
                for parent in parsed
                    .ontology
                    .hpo(phenotype.id.number())
                    .unwrap()
                    .all_parent_ids()
                {
                    // Keep the phenotype branch; inheritance/onset branches are not scored.
                    let term = parsed.ontology.hpo(parent).unwrap();
                    if parent.as_u32() == 118 || term.all_parent_ids().contains(&118u32.into()) {
                        closure.insert(HpoId::from_number(parent.as_u32()));
                    }
                }
            }
            if !closure.is_empty() {
                for term in &closure {
                    *counts.entry(term.clone()).or_default() += 1;
                }
                closures.insert(id.clone(), closure);
            }
        }
        let n = closures.len() as f64;
        if n == 0.0 {
            return Err(Error::Data("no positive HPO phenotype profiles".into()));
        }
        let weights = counts
            .into_iter()
            .map(|(id, count)| (id, -(count as f64 / n).ln()))
            .collect();
        Ok(Self {
            fingerprint,
            mondo,
            profiles,
            aliases,
            names,
            closures,
            weights,
            hpo_version: parsed.version.clone(),
            ontology: parsed,
            skipped_annotation_namespaces: records.skipped_namespaces,
        })
    }

    /// Content hash of all three source files, for persistent cache isolation.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// The underlying read-only HPO ontology, including direct parents and non-phenotype branches.
    pub fn ontology(&self) -> &hpo::Ontology {
        &self.ontology.ontology
    }

    /// Normalized alternative IDs and uniquely replaced obsolete IDs.
    pub fn hpo_aliases(&self) -> &BTreeMap<HpoId, HpoId> {
        &self.ontology.aliases
    }

    pub fn ontology_records(&self) -> &BTreeMap<HpoId, crate::OntologyTerm> {
        &self.ontology.records
    }

    pub fn disease_count(&self) -> usize {
        self.closures.len()
    }
    pub fn profiles(&self) -> impl Iterator<Item = &DiseaseProfile> {
        self.profiles.values()
    }
    pub fn profile(&self, id: &DiseaseId) -> Option<&DiseaseProfile> {
        self.profiles.get(self.aliases.get(id).unwrap_or(id))
    }

    /// Look up an active phenotype, normalizing alternative and uniquely replaced IDs.
    /// Root, inheritance, and onset terms are not phenotypic abnormalities.
    pub fn phenotype(&self, id: &HpoId) -> Option<Phenotype> {
        let id = self.ontology.aliases.get(id).unwrap_or(id);
        let term = self.ontology.ontology.hpo(id.number())?;
        term.all_parent_ids()
            .contains(&118u32.into())
            .then(|| Phenotype {
                id: id.clone(),
                name: term.name().into(),
            })
    }

    /// Resolve an HPO ID or label. Exact labels take precedence; otherwise returns
    /// substring candidates for the caller to disambiguate. Case and whitespace
    /// are normalized. Searches ontology labels, including unannotated terms.
    pub fn find_phenotypes(&self, query: &str) -> Vec<Phenotype> {
        let query = query.trim();
        if query.starts_with("HP:") {
            return query
                .parse()
                .ok()
                .and_then(|id| self.phenotype(&id))
                .into_iter()
                .collect();
        }
        let query = mondo::normalize_name(query);
        if query.is_empty() {
            return Vec::new();
        }
        let mut exact = Vec::new();
        let mut partial = Vec::new();
        for term in self.ontology.ontology.iter() {
            if !term.all_parent_ids().contains(&118u32.into()) {
                continue;
            }
            let name = mondo::normalize_name(term.name());
            if name.contains(&query) {
                let phenotype = Phenotype {
                    id: HpoId::from_number(term.id().as_u32()),
                    name: term.name().into(),
                };
                if name == query {
                    exact.push(phenotype);
                } else {
                    partial.push(phenotype);
                }
            }
        }
        let mut matches = if exact.is_empty() { partial } else { exact };
        matches.sort_by(|a, b| a.id.cmp(&b.id));
        matches
    }

    /// Whether an HPO term equals or descends from another term. Unknown or
    /// unresolved obsolete IDs return false; valid aliases are normalized.
    pub fn phenotype_is_a(&self, child: &HpoId, parent: &HpoId) -> bool {
        let child = self.ontology.aliases.get(child).unwrap_or(child);
        let parent = self.ontology.aliases.get(parent).unwrap_or(parent);
        self.ontology
            .ontology
            .hpo(child.number())
            .is_some_and(|term| {
                child == parent || term.all_parent_ids().contains(&parent.number().into())
            })
    }

    /// Rank every annotated disease using information-content-weighted ancestor Jaccard.
    /// Exact-equivalent records share a profile and cannot appear as self matches.
    pub fn similar(
        &self,
        source_id: &DiseaseId,
        options: &SimilarityOptions,
    ) -> Result<Vec<SimilarDisease>> {
        if !options.min_score.is_finite() || !(0.0..=1.0).contains(&options.min_score) {
            return Err(Error::InvalidRequest(
                "min_score must be finite and between 0 and 1".into(),
            ));
        }
        let source = self
            .profile(source_id)
            .ok_or_else(|| Error::Unmapped(source_id.to_string()))?;
        let a = self
            .closures
            .get(&source.id)
            .ok_or_else(|| Error::NoPhenotypes(source.id.clone()))?;
        let source_negative: BTreeSet<_> = source
            .excluded_phenotypes
            .iter()
            .map(|p| p.id.clone())
            .collect();
        let sum = |terms: &BTreeSet<HpoId>| terms.iter().map(|id| self.weights[id]).sum::<f64>();
        let a_weight = sum(a);
        if a_weight <= 0.0 {
            return Err(Error::Data(
                "source phenotype profile has no informative terms in this corpus".into(),
            ));
        }
        let mut ranked = Vec::new();
        for (id, b) in &self.closures {
            if id == &source.id {
                continue;
            }
            let candidate = &self.profiles[id];
            if candidate.phenotypes.len() < options.min_phenotypes {
                continue;
            }
            let overlap: f64 = a.intersection(b).map(|id| self.weights[id]).sum();
            let union = a_weight + sum(b) - overlap;
            let score = if union > 0.0 {
                (overlap / union).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if score <= 0.0 || score < options.min_score {
                continue;
            }
            let candidate_terms = term_ids(candidate);
            let candidate_negative: BTreeSet<_> = candidate
                .excluded_phenotypes
                .iter()
                .map(|p| p.id.clone())
                .collect();
            let mut conflicts = BTreeMap::new();
            for phenotype in &source.phenotypes {
                if candidate_negative.contains(&phenotype.id) {
                    conflicts.insert(phenotype.id.clone(), phenotype.clone());
                }
            }
            for phenotype in &candidate.phenotypes {
                if source_negative.contains(&phenotype.id) {
                    conflicts.insert(phenotype.id.clone(), phenotype.clone());
                }
            }
            ranked.push(SimilarDisease {
                disease_id: id.clone(),
                name: candidate.name.clone(),
                mondo: candidate.mondo.clone(),
                annotation_ids: candidate.annotation_ids.clone(),
                mesh_ids: candidate.mesh_ids.clone(),
                score,
                phenotype_count: candidate.phenotypes.len(),
                shared_phenotypes: source
                    .phenotypes
                    .iter()
                    .filter(|p| candidate_terms.contains(&p.id))
                    .cloned()
                    .collect(),
                source_only_phenotypes: source
                    .phenotypes
                    .iter()
                    .filter(|p| !candidate_terms.contains(&p.id))
                    .cloned()
                    .collect(),
                conflicting_phenotypes: conflicts.into_values().collect(),
            });
        }
        ranked.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.disease_id.cmp(&b.disease_id))
        });
        ranked.truncate(options.limit.get());
        Ok(ranked)
    }
}
