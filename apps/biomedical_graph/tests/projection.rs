use biomedical_graph::*;
use serde_json::json;

fn synthetic() -> pubtator3_hpo::Dataset {
    pubtator3_hpo::Dataset::from_readers(
        std::io::Cursor::new(include_str!("../../pubtator3-hpo/tests/fixtures/hp.obo")),
        include_str!("../../pubtator3-hpo/tests/fixtures/phenotype.hpoa").as_bytes(),
        include_str!("../../pubtator3-hpo/tests/fixtures/mondo.json").as_bytes(),
    )
    .unwrap()
}

#[test]
fn captured_pubtator_preserves_direction_and_document_scope() {
    let values: Vec<pubtator3::RelatedEntity> =
        serde_json::from_str(include_str!("fixtures/pubtator-relations.json")).unwrap();
    let graph = relations(&values, "official");
    let edge = graph
        .edges
        .values()
        .find(|e| e.kind == "PUBTATOR_RELATION" && e.target == "pubtator:@GENE_HTT")
        .unwrap();
    assert_eq!(edge.source, "pubtator:@DISEASE_Huntington_Disease");
    assert_eq!(edge.properties["relation_type"], "associate");
    assert_eq!(edge.properties["publications"], 3616);
    let docs = pubtator3::parse_documents(include_str!("fixtures/pubtator-document.json")).unwrap();
    let abstract_graph = documents(&docs, "abstract", "official");
    let full_graph = documents(&docs, "full_text", "official");
    let document_ids = |g: &GraphBatch| {
        g.nodes
            .values()
            .filter(|n| n.labels.iter().any(|l| l == "Document"))
            .map(|n| n.uid.clone())
            .collect::<Vec<_>>()
    };
    assert_ne!(document_ids(&abstract_graph), document_ids(&full_graph));
    assert_eq!(
        abstract_graph
            .nodes
            .values()
            .filter(|n| n.labels.iter().any(|l| l == "Mention"))
            .count(),
        26
    );
    assert!(
        abstract_graph
            .edges
            .values()
            .any(|e| e.kind == "DENOTES" && e.target == "pubtator:@GENE_ABCC1")
    );
    // Same local IDs from another upstream must not collide.
    assert_ne!(
        document_ids(&abstract_graph),
        document_ids(&documents(&docs, "abstract", "proxy"))
    );
    let mut other = docs[0].clone();
    other.id = "99999999".parse().unwrap();
    let two = documents(&[docs[0].clone(), other], "abstract", "official");
    assert_eq!(
        two.nodes
            .values()
            .filter(|n| n.labels.iter().any(|l| l == "Mention"))
            .count(),
        52
    );
}

#[test]
fn opaque_bioc_refs_do_not_create_false_mentions() {
    let docs = pubtator3::parse_documents(r#"{"id":"1","relations":[{"id":"r1","infons":{"type":"Positive_Correlation","score":"0.9","role1":{"accession":"@GENE_HTT","type":"Gene"},"role2":{"accession":"@DISEASE_Huntington_Disease","type":"Disease"}},"nodes":[{"refid":"display-entry","role":"gene"}]}]}"#).unwrap();
    let graph = documents(&docs, "abstract", "official");
    assert!(
        graph
            .nodes
            .values()
            .any(|n| n.labels.iter().any(|l| l == "BioCReference")
                && n.properties["refid"] == "display-entry")
    );
    assert!(
        !graph
            .nodes
            .values()
            .any(|n| n.labels.iter().any(|l| l == "Mention"))
    );
    let role1 = graph
        .edges
        .values()
        .find(|e| e.kind == "HAS_PARTICIPANT" && e.properties["role"] == "role1")
        .unwrap();
    assert_eq!(role1.target, "pubtator:@GENE_HTT");
}

#[test]
fn article_body_and_sentence_text_never_enter_graph_properties() {
    let value = json!({"id":"123", "pmid":123, "extra_body":"HIDDEN_ARTICLE_SENTINEL", "passages":[
        {"infons":{"type":"title"},"text":"Allowed title","offset":0},
        {"infons":{"section_type":"ABSTRACT"},"text":"Allowed abstract","offset":20},
        {"infons":{"section_type":"INTRO"},"text":"HIDDEN_ARTICLE_SENTINEL","offset":40,
         "sentences":[{"text":"HIDDEN_SENTENCE_SENTINEL","offset":40,
           "annotations":[{"id":"A1","text":"HTT","infons":{"type":"Gene","accession":"@GENE_HTT","extra_body":"HIDDEN_ARTICLE_SENTINEL"},"locations":[{"offset":41,"length":3}]}]}]}
    ]});
    let docs = pubtator3::parse_documents(&value.to_string()).unwrap();
    let graph = documents(&docs, "full_text", "official");
    let serialized = serde_json::to_string(&graph).unwrap();
    assert!(!serialized.contains("HIDDEN_ARTICLE_SENTINEL"));
    assert!(!serialized.contains("HIDDEN_SENTENCE_SENTINEL"));
    assert!(
        !graph
            .nodes
            .values()
            .any(|n| n.labels.iter().any(|l| l == "Passage" || l == "Sentence"))
    );
    let doc = graph
        .nodes
        .values()
        .find(|n| n.labels.iter().any(|l| l == "Document"))
        .unwrap();
    assert_eq!(doc.properties["title"], "Allowed title");
    assert_eq!(doc.properties["abstract"], "Allowed abstract");
    assert!(
        graph
            .edges
            .values()
            .any(|e| e.kind == "HAS_MENTION" && e.source == doc.uid)
    );
    assert!(
        graph.nodes.values().any(
            |n| n.labels.iter().any(|l| l == "MentionLocation") && n.properties["offset"] == 41
        )
    );
    let response:pubtator3::SearchResponse = serde_json::from_value(json!({
        "results":[{"pmid":123,"title":"Allowed title","text_hl":"HIDDEN_ARTICLE_SENTINEL","full_text":"HIDDEN_ARTICLE_SENTINEL"}],
        "count":1,"page_size":1,"current":1,"total_pages":1
    })).unwrap();
    assert!(
        !serde_json::to_string(&search(&response))
            .unwrap()
            .contains("HIDDEN_ARTICLE_SENTINEL")
    );
}

#[test]
fn hpo_curated_evidence_negative_features_and_direct_parents_are_preserved() {
    let dataset = synthetic();
    let id = "OMIM:100001".parse().unwrap();
    let disease = dataset.profile(&id).unwrap();
    let graph = profile(disease, &dataset);
    assert!(
        graph.edges.values().any(|e| e.kind == "IS_A"
            && e.source == "hpo:HP:0001002"
            && e.target == "hpo:HP:0001001")
    );
    assert!(
        !graph.edges.values().any(|e| e.kind == "IS_A"
            && e.source == "hpo:HP:0001002"
            && e.target == "hpo:HP:0000118")
    );
    assert!(graph.edges.values().any(|e| e.kind == "NORMALIZES_TO"
        && e.source == "hpo:HP:0001006"
        && e.target == "hpo:HP:0001004"));
    assert!(graph.edges.values().any(|e| e.kind == "REPLACED_BY"
        && e.source == "hpo:HP:0001008"
        && e.target == "hpo:HP:0001002"));
    assert!(
        graph
            .edges
            .values()
            .any(|e| e.kind == "EXCLUDES_PHENOTYPE" && e.target == "hpo:HP:0001003")
    );
    assert!(
        !graph
            .edges
            .values()
            .any(|e| e.kind == "HAS_PHENOTYPE" && e.target == "hpo:HP:0001003")
    );
    assert!(
        graph
            .edges
            .values()
            .any(|e| e.kind == "SUPPORTED_BY" && e.target == "publication:123456")
    );
    assert_eq!(
        graph
            .nodes
            .values()
            .filter(|n| n.labels.iter().any(|l| l == "HpoAnnotation"))
            .count(),
        disease.annotations.len()
    );
    let again = profile(disease, &dataset);
    assert_eq!(
        serde_json::to_value(graph).unwrap(),
        serde_json::to_value(again).unwrap()
    );
}

#[test]
fn snapshot_and_mapping_configuration_isolate_cache_identity() {
    let dataset = synthetic();
    let first_hash = dataset.fingerprint().to_owned();
    let client = pubtator3_hpo::Client::new(pubtator3::Client::new().unwrap(), dataset);
    assert_ne!(
        client.cache_identity(),
        client.clone().allow_name_fallback(false).cache_identity()
    );
    assert_ne!(
        client.cache_identity(),
        client
            .clone()
            .with_override("D000001".parse().unwrap(), "OMIM:100002".parse().unwrap())
            .unwrap()
            .cache_identity()
    );
    let alternate = pubtator3_hpo::Dataset::from_readers(
        std::io::Cursor::new(format!(
            "{}\n! Different snapshot\n",
            include_str!("../../pubtator3-hpo/tests/fixtures/hp.obo")
        )),
        include_str!("../../pubtator3-hpo/tests/fixtures/phenotype.hpoa").as_bytes(),
        include_str!("../../pubtator3-hpo/tests/fixtures/mondo.json").as_bytes(),
    )
    .unwrap();
    assert_ne!(first_hash, alternate.fingerprint());
}

#[test]
fn official_hpo_subset_preserves_definition_and_exact_mapping() {
    let dataset =
        pubtator3_hpo::Dataset::from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))
            .unwrap();
    let disease = dataset.profile(&"OMIM:143100".parse().unwrap()).unwrap();
    assert_eq!(disease.id.to_string(), "MONDO:0007739");
    let graph = profile(disease, &dataset);
    let chorea = &graph.nodes["hpo:HP:0002072"];
    assert_eq!(chorea.properties["name"], "Chorea");
    assert!(chorea.properties["tags"]["def"].is_array());
    assert_eq!(
        graph
            .edges
            .values()
            .filter(|e| e.kind == "HAS_ANNOTATION")
            .count(),
        4
    );
    let _extensible = GraphBatch::default().node(
        "future:1",
        "ClinicalTrial",
        Properties::from([("phase".into(), json!(2))]),
    );
}
