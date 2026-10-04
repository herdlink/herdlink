use crate::{
    GraphBatch,
    model::{hash, props},
};
use pubtator3::{
    Annotation, BioCRelation, Document, Entity, EntityInfo, MeshSynonyms, RelatedEntity,
    SearchResponse,
};
use pubtator3_hpo::{Dataset, DiseaseProfile, HpoId, MappedDisease};
use serde_json::json;
use std::collections::BTreeSet;

fn entity_label(biotype: &str) -> &'static str {
    match biotype.to_ascii_lowercase().as_str() {
        "gene" => "Gene",
        "disease" => "DiseaseEntity",
        "chemical" => "Chemical",
        "variant" | "mutation" => "Variant",
        "species" => "Species",
        "cellline" | "cell_line" => "CellLine",
        _ => "OtherEntity",
    }
}
fn identifier(batch: &mut GraphBatch, database: &str, id: &str) -> String {
    batch.node(
        format!("identifier:{database}:{id}"),
        "Identifier",
        props(json!({"database":database,"id":id})),
    )
}
fn entity_stub(batch: &mut GraphBatch, id: &pubtator3::EntityId) -> String {
    let uid = batch.node(
        format!("pubtator:{id}"),
        "Entity",
        props(json!({"accession":id.to_string()})),
    );
    batch.node(&uid, entity_label(id.namespace()), Default::default());
    uid
}
pub fn entities(values: &[Entity]) -> GraphBatch {
    let mut batch = GraphBatch::default();
    for value in values {
        let uid = entity_stub(&mut batch, &value.id);
        batch.node(&uid, entity_label(&value.biotype), props(value));
        if let (Some(db), Some(id)) = (&value.db, &value.db_id) {
            let target = identifier(&mut batch, db, id);
            batch.edge(&uid, "IDENTIFIED_BY", &target, "", Default::default());
        }
    }
    batch
}
pub fn relations(values: &[RelatedEntity], namespace: &str) -> GraphBatch {
    let mut batch = GraphBatch::default();
    for value in values {
        let source = entity_stub(&mut batch, &value.source);
        let target = entity_stub(&mut batch, &value.target);
        let uid = batch.node(
            format!("relation-summary:{}", hash(&(namespace, value))),
            "RelationSummary",
            props(value),
        );
        batch.edge(&uid, "SOURCE", &source, "", Default::default());
        batch.edge(&uid, "TARGET", &target, "", Default::default());
        // Preserve endpoint orientation even for conceptually symmetric relation types.
        batch.edge(&source,"PUBTATOR_RELATION",&target,&uid,props(json!({"relation_type":value.relation_type,"publications":value.publications,"summary_uid":uid})));
    }
    batch
}
pub fn search(value: &SearchResponse) -> GraphBatch {
    let mut batch = GraphBatch::default();
    for publication in &value.results {
        // An allowlist prevents unknown fields from smuggling article text into the graph.
        let properties = props(json!({"pmid":publication.pmid,"pmcid":publication.pmcid,
            "title":publication.title,"journal":publication.journal,"authors":publication.authors,
            "date":publication.date,"doi":publication.doi,"citations":publication.citations}));
        let uid = batch.node(
            format!("publication:{}", publication.pmid),
            "Publication",
            properties,
        );
        if let Some(pmcid) = &publication.pmcid {
            let id = identifier(&mut batch, "PMC", pmcid.as_ref());
            batch.edge(&uid, "IDENTIFIED_BY", &id, "", Default::default());
        }
        if let Some(doi) = &publication.doi {
            let id = identifier(&mut batch, "DOI", doi);
            batch.edge(&uid, "IDENTIFIED_BY", &id, "", Default::default());
        }
    }
    batch
}
pub fn mesh_terms(value: &MeshSynonyms) -> GraphBatch {
    let mut batch = GraphBatch::default();
    let descriptor = identifier(&mut batch, "ncbi_mesh", value.descriptor.as_ref());
    batch.node(
        &descriptor,
        "MeshDescriptor",
        props(json!({"metadata_json":serde_json::to_string(&value.extra).unwrap()})),
    );
    for term in &value.terms {
        let uid = batch.node(format!("mesh-term:{}", term.id), "MeshTerm", props(term));
        batch.edge(
            &descriptor,
            "HAS_ENTRY_TERM",
            &uid,
            "",
            props(json!({"preferred":term.preferred})),
        );
    }
    batch
}
fn info_entity(batch: &mut GraphBatch, info: &EntityInfo) -> Option<String> {
    if let Some(accession) = &info.accession {
        let uid = entity_stub(batch, accession);
        if let Some(kind) = &info.entity_type {
            batch.node(&uid, entity_label(kind), Default::default());
        }
        if let Some(name) = &info.name {
            batch.node(&uid, "Entity", props(json!({"name":name})));
        }
        Some(uid)
    } else {
        // Identifier strings can contain several database IDs. Keep the opaque value intact.
        info.identifier
            .as_ref()
            .map(|id| identifier(batch, info.database.as_deref().unwrap_or("unspecified"), id))
    }
}
fn annotation(batch: &mut GraphBatch, doc: &str, owner: &str, value: &Annotation) {
    let uid = batch.node(
        format!("{doc}:annotation:{}", value.id),
        "Mention",
        props(
            json!({"id":value.id,"text":value.text,"type":value.infons.entity_type,
            "accession":value.infons.accession,"identifier":value.infons.identifier,
            "name":value.infons.name,"database":value.infons.database,
            "biotype":value.infons.biotype,"valid":value.infons.valid}),
        ),
    );
    batch.edge(owner, "HAS_MENTION", &uid, "", Default::default());
    if let Some(entity) = info_entity(batch, &value.infons) {
        batch.edge(&uid, "DENOTES", &entity, "", Default::default());
    }
    for (index, location) in value.locations.iter().enumerate() {
        let loc = batch.node(
            format!("{uid}:location:{index}"),
            "MentionLocation",
            props(location),
        );
        batch.edge(&uid, "HAS_LOCATION", &loc, "", Default::default());
    }
}
fn bioc_relation(batch: &mut GraphBatch, doc: &str, owner: &str, value: &BioCRelation) {
    let uid = batch.node(
        format!("{owner}:relation:{}", value.id),
        "ExtractedRelation",
        props(json!({"id":value.id,"type":value.infons.relation_type,
            "score":value.infons.score,"container":owner})),
    );
    batch.edge(doc, "HAS_RELATION", &uid, "", Default::default());
    for (role, info) in [
        ("role1", &value.infons.role1),
        ("role2", &value.infons.role2),
    ] {
        if let Some(entity) = info.as_ref().and_then(|info| info_entity(batch, info)) {
            batch.edge(
                &uid,
                "HAS_PARTICIPANT",
                &entity,
                role,
                props(json!({"role":role})),
            );
        }
    }
    for (index, reference) in value.nodes.iter().enumerate() {
        // refid is opaque, not guaranteed to be an annotation ID. Never invent a mention link.
        let node = batch.node(
            format!("{uid}:ref:{index}"),
            "BioCReference",
            props(json!({"refid":reference.refid,"role":reference.role})),
        );
        batch.edge(
            &uid,
            "HAS_REFERENCE",
            &node,
            "",
            props(json!({"document_uid":doc})),
        );
    }
}
/// Persist bibliographic text and upstream structured annotations, without passage/sentence nodes.
pub fn documents(values: &[Document], scope: &str, namespace: &str) -> GraphBatch {
    let mut batch = GraphBatch::default();
    for value in values {
        let title = document_text(value, "title");
        let abstract_text = document_text(value, "abstract");
        let doc = batch.node(
            format!("document:{}", hash(&(namespace, scope, value))),
            "Document",
            props(json!({
                "local_id":value.id.to_string(),"scope":scope,"namespace":namespace,
                "title":title,"abstract":abstract_text
            })),
        );
        if let Some(pmid) = value.pmid {
            let publication = batch.node(
                format!("publication:{pmid}"),
                "Publication",
                props(json!({"pmid":pmid,"title":title,"abstract":abstract_text})),
            );
            batch.edge(&publication, "HAS_DOCUMENT", &doc, "", Default::default());
        }
        if let Some(pmcid) = &value.pmcid {
            let id = identifier(&mut batch, "PMC", pmcid.as_ref());
            batch.edge(&doc, "IDENTIFIED_BY", &id, "", Default::default());
        }
        for (pi, passage) in value.passages.iter().enumerate() {
            let p = format!("{doc}:passage:{pi}");
            for ann in &passage.annotations {
                annotation(&mut batch, &doc, &doc, ann);
            }
            for rel in &passage.relations {
                bioc_relation(&mut batch, &doc, &p, rel);
            }
            for (si, sentence) in passage.sentences.iter().enumerate() {
                let s = format!("{p}:sentence:{si}");
                for ann in &sentence.annotations {
                    annotation(&mut batch, &doc, &doc, ann);
                }
                for rel in &sentence.relations {
                    bioc_relation(&mut batch, &doc, &s, rel);
                }
            }
        }
        for rel in &value.relations {
            bioc_relation(&mut batch, &doc, &doc, rel);
        }
    }
    batch
}

/// Only explicitly classified title/abstract passages are article content we retain.
fn document_text(document: &Document, section: &str) -> Option<String> {
    let parts: Vec<_> = document
        .passages
        .iter()
        .filter(|p| {
            ["type", "section_type"].iter().any(|key| {
                p.infons
                    .get(*key)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|s| s.eq_ignore_ascii_case(section))
            })
        })
        .filter_map(|p| p.text.as_deref())
        .filter(|s| !s.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn hpo_terms(
    batch: &mut GraphBatch,
    dataset: &Dataset,
    seeds: impl IntoIterator<Item = HpoId>,
    snapshot: &str,
) {
    let mut seen = BTreeSet::new();
    let mut pending: Vec<_> = seeds.into_iter().collect();
    while let Some(id) = pending.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let uid = batch.node(
            format!("hpo:{id}"),
            "HpoTerm",
            props(json!({"id":id.to_string()})),
        );
        if let Some(record) = dataset.ontology_records().get(&id) {
            batch.node(&uid, "HpoTerm", props(record));
            for replacement in &record.replacements {
                let target = batch.node(
                    format!("hpo:{replacement}"),
                    "HpoTerm",
                    props(json!({"id":replacement.to_string()})),
                );
                batch.edge(
                    &uid,
                    "REPLACED_BY",
                    &target,
                    snapshot,
                    props(json!({"snapshot_uid":snapshot})),
                );
                pending.push(replacement.clone());
            }
        }
        if let Some(canonical) = dataset.hpo_aliases().get(&id) {
            let target = batch.node(
                format!("hpo:{canonical}"),
                "HpoTerm",
                props(json!({"id":canonical.to_string()})),
            );
            batch.node(&uid, "HpoAlias", Default::default());
            batch.edge(
                &uid,
                "NORMALIZES_TO",
                &target,
                snapshot,
                props(json!({"snapshot_uid":snapshot})),
            );
            pending.push(canonical.clone());
            continue;
        }
        let Some(term) = dataset.ontology().hpo(id.number()) else {
            continue;
        };
        batch.node(&uid, "HpoTerm", props(json!({"name":term.name()})));
        for (alias, canonical) in dataset.hpo_aliases() {
            if canonical == &id {
                pending.push(alias.clone());
                let a = batch.node(
                    format!("hpo:{alias}"),
                    "HpoAlias",
                    props(json!({"id":alias.to_string()})),
                );
                batch.node(&a, "HpoTerm", Default::default());
                if let Some(record) = dataset.ontology_records().get(alias) {
                    batch.node(&a, "HpoAlias", props(record));
                }
                batch.edge(
                    &a,
                    "NORMALIZES_TO",
                    &uid,
                    snapshot,
                    props(json!({"snapshot_uid":snapshot})),
                );
            }
        }
        batch.edge(&uid, "IN_SNAPSHOT", snapshot, "", Default::default());
        for parent in term.parent_ids() {
            let parent_id: HpoId = parent.to_string().parse().expect("valid ontology ID");
            let target = batch.node(
                format!("hpo:{parent_id}"),
                "HpoTerm",
                props(json!({"id":parent_id.to_string()})),
            );
            batch.edge(
                &uid,
                "IS_A",
                &target,
                snapshot,
                props(json!({"snapshot_uid":snapshot})),
            );
            pending.push(parent_id);
        }
    }
}
/// Project selected ontology terms and their direct-parent closure, including OBO metadata.
pub fn ontology_terms(dataset: &Dataset, seeds: impl IntoIterator<Item = HpoId>) -> GraphBatch {
    let mut batch = GraphBatch::default();
    let snapshot = batch.node(
        format!("hpo-snapshot:{}", dataset.fingerprint()),
        "DatasetSnapshot",
        props(json!({"fingerprint":dataset.fingerprint(),"hpo_version":dataset.hpo_version})),
    );
    hpo_terms(&mut batch, dataset, seeds, &snapshot);
    batch
}
pub fn profile(value: &DiseaseProfile, dataset: &Dataset) -> GraphBatch {
    let mut batch = GraphBatch::default();
    let snapshot = batch.node(
        format!("hpo-snapshot:{}", dataset.fingerprint()),
        "DatasetSnapshot",
        props(json!({"fingerprint":dataset.fingerprint(),"hpo_version":dataset.hpo_version})),
    );
    let disease = batch.node(
        format!("disease:{}", value.id),
        "Disease",
        props(json!({"id":value.id.to_string(),"name":value.name,"mondo":value.mondo})),
    );
    let p = batch.node(
        format!("{snapshot}:profile:{}", value.id),
        "HpoDiseaseProfile",
        props(value),
    );
    batch.edge(&p, "PROFILE_OF", &disease, "", Default::default());
    batch.edge(&p, "IN_SNAPSHOT", &snapshot, "", Default::default());
    for record_id in &value.annotation_ids {
        let record = batch.node(
            format!("disease-record:{record_id}"),
            "DiseaseRecord",
            props(json!({"id":record_id.to_string()})),
        );
        batch.edge(&p, "HAS_RECORD", &record, "", Default::default());
        batch.edge(
            &record,
            "RECORD_OF",
            &disease,
            &snapshot,
            props(json!({"snapshot_uid":snapshot})),
        );
    }
    for mesh in &value.mesh_ids {
        let id = identifier(&mut batch, "ncbi_mesh", &mesh.to_string());
        batch.edge(
            &disease,
            "EXACT_MESH_MAPPING",
            &id,
            &snapshot,
            props(json!({"snapshot_uid":snapshot})),
        );
    }
    let mut seeds = Vec::new();
    for (index, value) in value.annotations.iter().enumerate() {
        let a = batch.node(
            format!("{snapshot}:hpo-annotation:{}", hash(&(&p, index, value))),
            "HpoAnnotation",
            props(value),
        );
        let record = batch.node(
            format!("disease-record:{}", value.disease_id),
            "DiseaseRecord",
            props(json!({"id":value.disease_id.to_string()})),
        );
        let term = batch.node(
            format!("hpo:{}", value.phenotype),
            "HpoTerm",
            props(json!({"id":value.phenotype.to_string()})),
        );
        batch.edge(&p, "HAS_ANNOTATION", &a, "", Default::default());
        batch.edge(&a, "ANNOTATES_RECORD", &record, "", Default::default());
        batch.edge(&a, "PHENOTYPE", &term, "", Default::default());
        seeds.push(value.phenotype.clone());
        for (kind, raw) in [
            ("ONSET", &value.onset),
            ("FREQUENCY", &value.frequency),
            ("MODIFIER", &value.modifier),
        ] {
            for id in raw
                .iter()
                .flat_map(|v| v.split(';'))
                .filter_map(|v| v.trim().parse::<HpoId>().ok())
            {
                let target = batch.node(
                    format!("hpo:{id}"),
                    "HpoTerm",
                    props(json!({"id":id.to_string()})),
                );
                batch.edge(&a, kind, &target, "", Default::default());
                seeds.push(id);
            }
        }
        for reference in value.reference.split(';').map(str::trim) {
            let target = if let Some(pmid) = reference
                .strip_prefix("PMID:")
                .and_then(|v| v.parse::<pubtator3::Pmid>().ok())
            {
                batch.node(
                    format!("publication:{pmid}"),
                    "Publication",
                    props(json!({"pmid":pmid})),
                )
            } else {
                identifier(&mut batch, "reference", reference)
            };
            batch.edge(&a, "SUPPORTED_BY", &target, "", Default::default());
        }
    }
    for (kind, features) in [
        ("HAS_PHENOTYPE", &value.phenotypes),
        ("EXCLUDES_PHENOTYPE", &value.excluded_phenotypes),
        ("CONFLICTING_PHENOTYPE", &value.conflicting_phenotypes),
    ] {
        for feature in features {
            let term = batch.node(format!("hpo:{}", feature.id), "HpoTerm", props(feature));
            batch.edge(&p, kind, &term, "", Default::default());
            seeds.push(feature.id.clone());
        }
    }
    hpo_terms(&mut batch, dataset, seeds, &snapshot);
    batch
}
pub fn mapped_disease(value: &MappedDisease, dataset: &Dataset, configuration: &str) -> GraphBatch {
    let mut batch = profile(&value.profile, dataset);
    batch.extend(entities(std::slice::from_ref(&value.entity)));
    let uid = batch.node(
        format!(
            "mapping:{}",
            hash(&(configuration, &value.entity.id, &value.mapping))
        ),
        "DiseaseMapping",
        props(&value.mapping),
    );
    batch.node(
        &uid,
        "DiseaseMapping",
        props(json!({"configuration":configuration})),
    );
    batch.edge(
        &format!("pubtator:{}", value.entity.id),
        "MAPPED_VIA",
        &uid,
        "",
        Default::default(),
    );
    batch.edge(
        &uid,
        "MAPS_TO",
        &format!("disease:{}", value.mapping.disease_id),
        "",
        Default::default(),
    );
    batch
}
