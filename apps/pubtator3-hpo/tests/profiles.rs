use pubtator3_hpo::*;
use std::io::Cursor;

const HP: &str = include_str!("fixtures/hp.obo");
const ANNOTATIONS: &str = include_str!("fixtures/phenotype.hpoa");
const MONDO: &str = include_str!("fixtures/mondo.json");

fn dataset() -> Dataset {
    Dataset::from_readers(Cursor::new(HP), ANNOTATIONS.as_bytes(), MONDO.as_bytes()).unwrap()
}

fn entity(name: &str, mesh: &str) -> pubtator3::Entity {
    serde_json::from_value(serde_json::json!({
        "_id": "@DISEASE_Test", "name": name, "biotype": "disease",
        "db": "ncbi_mesh", "db_id": mesh
    }))
    .unwrap()
}

#[test]
fn ids_are_typed_and_validated_including_serde() {
    assert!("D006816".parse::<MeshId>().is_ok());
    assert!("C000726747".parse::<MeshId>().is_ok());
    assert!("C564296".parse::<MeshId>().is_ok());
    for value in ["", "D0", "D000000", "D006816x", "143100", "MESH:D006816"] {
        assert!(value.parse::<MeshId>().is_err(), "{value}");
    }
    assert_eq!(
        "MIM:143100".parse::<DiseaseId>().unwrap().as_str(),
        "OMIM:143100"
    );
    assert_eq!(
        "Orphanet:399".parse::<DiseaseId>().unwrap().as_str(),
        "ORPHA:399"
    );
    for value in ["HP:0000000", "HP:118", "HP:0000118x"] {
        assert!(serde_json::from_value::<HpoId>(value.into()).is_err());
    }
    assert!("MONDO:0007739".parse::<MondoId>().is_ok());
    assert!("OMIM:123".parse::<DiseaseId>().is_err());
    assert!("OMIM:000000".parse::<DiseaseId>().is_err());
    assert!("ORPHA:-1".parse::<DiseaseId>().is_err());
}

#[test]
fn exact_ids_win_over_names_and_equivalent_records_merge() {
    let ds = dataset();
    let mapping = ds
        .mapper()
        .map_entity(&entity("Sibling Disease", "D000001"))
        .unwrap()
        .require_unique()
        .unwrap();
    assert_eq!(mapping.method, MappingMethod::ExactMesh);
    assert_eq!(mapping.disease_id.as_str(), "MONDO:0000001");
    assert_eq!(mapping.annotation_ids.len(), 2);
    let mesh = ds
        .mapper()
        .map_mesh(&"C000000001".parse().unwrap())
        .require_unique()
        .unwrap();
    assert_eq!(mesh.disease_id, mapping.disease_id);
    let omim = ds.profile(&"OMIM:100001".parse().unwrap()).unwrap();
    let orpha = ds.profile(&"ORPHA:1".parse().unwrap()).unwrap();
    assert_eq!(omim.id, orpha.id);
    assert_eq!(ds.disease_count(), 7);
    assert_eq!(omim.phenotypes.len(), 2);
    assert_eq!(omim.phenotypes[0].id.as_str(), "HP:0001002");
    assert_eq!(omim.phenotypes[1].id.as_str(), "HP:0001004");
    assert_eq!(
        ds.mapper()
            .map_id(&"ORPHA:1".parse().unwrap())
            .require_unique()
            .unwrap()
            .method,
        MappingMethod::AnnotationId
    );
}

#[test]
fn fallback_is_conservative_optional_and_auditable() {
    let ds = dataset();
    let e = entity("  SOURCE   alias ", "C000000009");
    let mapping = ds
        .mapper()
        .map_entity(&e)
        .unwrap()
        .require_unique()
        .unwrap();
    assert_eq!(mapping.method, MappingMethod::ExactName);
    assert_eq!(mapping.disease_id.as_str(), "MONDO:0000001");
    assert!(matches!(
        ds.mapper()
            .allow_name_fallback(false)
            .map_entity(&e)
            .unwrap(),
        MappingOutcome::Unmapped { .. }
    ));
    for name in ["Related-only alias", "Source", "Source Disease syndrome"] {
        assert!(matches!(
            ds.mapper().map_name(name),
            MappingOutcome::Unmapped { .. }
        ));
    }
    assert!(
        matches!(
            ds.mapper().map_id(&"OMIM:999999".parse().unwrap()),
            MappingOutcome::Unmapped { .. }
        ),
        "plain Mondo xrefs must not imply equivalence"
    );
    let override_mapping = ds
        .mapper()
        .allow_name_fallback(false)
        .with_override(
            "C000000009".parse().unwrap(),
            "OMIM:100001".parse().unwrap(),
        )
        .unwrap()
        .map_entity(&e)
        .unwrap()
        .require_unique()
        .unwrap();
    assert_eq!(override_mapping.method, MappingMethod::Override);
    assert!(
        ds.mapper()
            .with_override("D000009".parse().unwrap(), "OMIM:999999".parse().unwrap())
            .is_err()
    );
}

#[test]
fn ambiguity_and_missing_annotations_do_not_become_name_guesses() {
    let ds = dataset();
    let ambiguous = ds
        .mapper()
        .map_entity(&entity("Source Disease", "D000099"))
        .unwrap();
    match &ambiguous {
        MappingOutcome::Ambiguous(candidates) => assert_eq!(candidates.len(), 2),
        other => panic!("expected ambiguous MeSH equivalence: {other:?}"),
    }
    assert!(matches!(
        ambiguous.require_unique(),
        Err(Error::AmbiguousMapping(_))
    ));
    assert!(matches!(
        ds.mapper().map_name("Shared Disease"),
        MappingOutcome::Ambiguous(_)
    ));
    assert!(
        matches!(
            ds.mapper()
                .map_entity(&entity("Source Disease", "D000008"))
                .unwrap(),
            MappingOutcome::Unmapped { .. }
        ),
        "known ID without HPO annotations must stay unmapped"
    );
    let mut gene = entity("Source Disease", "D000001");
    gene.id = "@GENE_Test".parse().unwrap();
    gene.biotype = "gene".into();
    assert!(ds.mapper().map_entity(&gene).is_err());
}

#[test]
fn negative_features_metadata_and_aliases_are_preserved() {
    let ds = dataset();
    let source = ds.profile(&"OMIM:100001".parse().unwrap()).unwrap();
    assert_eq!(source.unscored_annotations, 2); // inheritance + missing term
    assert_eq!(source.excluded_phenotypes.len(), 2);
    assert!(source.conflicting_phenotypes.is_empty());
    assert!(
        source
            .annotations
            .iter()
            .any(|a| a.frequency.as_deref() == Some("80%")
                && a.onset.as_deref() == Some("HP:0003596")
                && a.reference == "PMID:123456")
    );
    let excluded: Vec<_> = source.annotations.iter().filter(|a| a.excluded).collect();
    assert_eq!(excluded.len(), 5); // NOT, excluded frequency class, 0%, 0/n, other NOT
    assert!(
        source
            .annotations
            .iter()
            .all(|a| a.phenotype.as_str() != "HP:0001006" && a.phenotype.as_str() != "HP:0001008")
    );
    let twin = ds.profile(&"OMIM:100002".parse().unwrap()).unwrap();
    assert_eq!(twin.conflicting_phenotypes[0].id.as_str(), "HP:0001002");
    assert_eq!(ds.skipped_annotation_namespaces, 1);
}

#[test]
fn semantic_similarity_ranks_related_features_and_deduplicates_self() {
    let ds = dataset();
    let ranked = ds
        .similar(&"ORPHA:1".parse().unwrap(), &SimilarityOptions::default())
        .unwrap();
    assert_eq!(ranked.len(), 3);
    assert_eq!(ranked[0].disease_id.as_str(), "MONDO:0000002");
    assert!((ranked[0].score - 1.0).abs() < 1e-12);
    assert_eq!(ranked[0].shared_phenotypes.len(), 2);
    assert_eq!(ranked[0].conflicting_phenotypes.len(), 1);
    assert_eq!(ranked[1].disease_id.as_str(), "MONDO:0000005");
    // Seven canonical profiles: four contain each neurologic ancestor, three
    // contain A, two contain C. Equivalent annotation records count only once.
    let shared_weight = 2.0 * (7.0_f64 / 4.0).ln() + (7.0_f64 / 3.0).ln();
    let expected = shared_weight / (shared_weight + (7.0_f64 / 2.0).ln());
    assert!((ranked[1].score - expected).abs() < 1e-12);
    let sibling = &ranked[2];
    assert_eq!(sibling.disease_id.as_str(), "MONDO:0000003");
    assert!(sibling.score > 0.0 && sibling.score < ranked[1].score);
    assert!(
        sibling.shared_phenotypes.is_empty(),
        "siblings can match via shared ancestors"
    );
    assert_eq!(sibling.conflicting_phenotypes[0].id.as_str(), "HP:0001003");
    assert_eq!(sibling.source_only_phenotypes.len(), 2);
    assert!(
        ranked
            .iter()
            .all(|d| d.disease_id.as_str() != "MONDO:0000001")
    );
    let options = SimilarityOptions {
        limit: 1.try_into().unwrap(),
        min_phenotypes: 2,
        min_score: 0.9,
    };
    assert_eq!(
        ds.similar(&"OMIM:100001".parse().unwrap(), &options)
            .unwrap()
            .len(),
        1
    );
    for min_score in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert!(
            ds.similar(
                &"OMIM:100001".parse().unwrap(),
                &SimilarityOptions {
                    min_score,
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
    let tied = ds
        .similar(&"OMIM:100004".parse().unwrap(), &Default::default())
        .unwrap();
    assert_eq!(tied[0].disease_id.as_str(), "MONDO:0000006");
    assert_eq!(tied[1].disease_id.as_str(), "MONDO:0000007");
}

#[test]
fn phenotype_lookup_resolves_labels_aliases_and_ambiguous_candidates() {
    let ds = dataset();
    let exact = ds.find_phenotypes("  FEATURE   A ");
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].id.as_str(), "HP:0001002");
    let partial = ds.find_phenotypes("feature");
    assert_eq!(partial.len(), 6);
    assert!(partial.windows(2).all(|w| w[0].id < w[1].id));
    // Includes parent labels with no direct disease annotations.
    assert_eq!(
        ds.find_phenotypes("Movement feature")[0].id.as_str(),
        "HP:0001001"
    );
    assert_eq!(
        ds.find_phenotypes("HP:0001006")[0].id.as_str(),
        "HP:0001004"
    );
    assert_eq!(
        ds.find_phenotypes("HP:0001008")[0].id.as_str(),
        "HP:0001002"
    );
    for query in [
        "",
        " ",
        "unknown symptom",
        "HP:invalid",
        "HP:9999999",
        "HP:0000118",
        "Mode of inheritance",
    ] {
        assert!(ds.find_phenotypes(query).is_empty(), "{query}");
    }
    assert!(ds.phenotype_is_a(
        &"HP:0001002".parse().unwrap(),
        &"HP:0001001".parse().unwrap()
    ));
    assert!(ds.phenotype_is_a(
        &"HP:0001008".parse().unwrap(),
        &"HP:0001002".parse().unwrap()
    ));
    assert!(ds.phenotype_is_a(
        &"HP:0001006".parse().unwrap(),
        &"HP:0001004".parse().unwrap()
    ));
    assert!(!ds.phenotype_is_a(
        &"HP:0001003".parse().unwrap(),
        &"HP:0001002".parse().unwrap()
    ));
    assert!(!ds.phenotype_is_a(
        &"HP:9999999".parse().unwrap(),
        &"HP:9999999".parse().unwrap()
    ));
}

#[test]
fn absent_only_and_uninformative_profiles_have_explicit_errors() {
    let mut annotations = ANNOTATIONS.to_owned();
    annotations.push_str(
        "OMIM:100009\tAbsent-only disease\tNOT\tHP:0001002\tPMID:1\tTAS\t\t\t\t\tP\tTEST\n",
    );
    let ds =
        Dataset::from_readers(Cursor::new(HP), annotations.as_bytes(), MONDO.as_bytes()).unwrap();
    assert!(matches!(
        ds.similar(&"OMIM:100009".parse().unwrap(), &Default::default()),
        Err(Error::NoPhenotypes(_))
    ));
    let uniform = ANNOTATIONS
        .lines()
        .filter(|line| !line.starts_with("OTHER:"))
        .map(|line| {
            if line.starts_with('#') || line.starts_with("database_id") {
                line.to_owned()
            } else {
                let mut fields: Vec<_> = line.split('\t').collect();
                fields[2] = "";
                fields[3] = "HP:0001002";
                fields[7] = "";
                fields[10] = "P";
                fields.join("\t")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let ds = Dataset::from_readers(Cursor::new(HP), uniform.as_bytes(), MONDO.as_bytes()).unwrap();
    assert!(
        matches!(ds.similar(&"OMIM:100001".parse().unwrap(), &Default::default()),
        Err(Error::Data(message)) if message.contains("no informative terms"))
    );
}

#[test]
fn malformed_graphs_and_annotations_return_errors() {
    let cycle = HP.replace(
        "is_a: HP:0001001 ! Movement feature",
        "is_a: HP:0001002 ! Cyclic parent",
    );
    assert!(matches!(
        Dataset::from_readers(Cursor::new(cycle), ANNOTATIONS.as_bytes(), MONDO.as_bytes()),
        Err(Error::Data(_))
    ));
    let missing = HP.replace("is_a: HP:0000001 ! All", "is_a: HP:9999999 ! Missing");
    assert!(
        Dataset::from_readers(
            Cursor::new(missing),
            ANNOTATIONS.as_bytes(),
            MONDO.as_bytes()
        )
        .is_err()
    );
    let invalid = ANNOTATIONS.replace("\tNOT\t", "\tMAYBE\t");
    assert!(Dataset::from_readers(Cursor::new(HP), invalid.as_bytes(), MONDO.as_bytes()).is_err());
    let invalid = ANNOTATIONS.replace("OMIM:100001", "OMIM:bogus");
    assert!(Dataset::from_readers(Cursor::new(HP), invalid.as_bytes(), MONDO.as_bytes()).is_err());
}

#[test]
fn real_release_metadata_maps_huntington_and_exposes_canvas_mesh_gap() {
    // Mondo nodes are captured from v2026-09-01. Phenotypes are synthetic test inputs.
    let annotations = ANNOTATIONS
        .replace("OMIM:100001", "OMIM:143100")
        .replace("ORPHA:1\t", "ORPHA:399\t")
        .replace("OMIM:100002", "OMIM:614575");
    let ds = Dataset::from_readers(
        Cursor::new(HP),
        annotations.as_bytes(),
        include_bytes!("fixtures/mondo-real.json").as_slice(),
    )
    .unwrap();
    let huntington = ds
        .mapper()
        .map_mesh(&"D006816".parse().unwrap())
        .require_unique()
        .unwrap();
    assert_eq!(huntington.disease_id.as_str(), "MONDO:0007739");
    let canvas = ds.mapper().map_name("CANVAS").require_unique().unwrap();
    assert_eq!(canvas.disease_id.as_str(), "MONDO:0044720");
    assert_eq!(canvas.annotation_ids[0].as_str(), "OMIM:614575");
    assert!(matches!(
        ds.mapper().map_mesh(&"C000726747".parse().unwrap()),
        MappingOutcome::Unmapped { .. }
    ));
    assert!(
        matches!(
            ds.mapper().map_id(&"OMIM:608088".parse().unwrap()),
            MappingOutcome::Unmapped { .. }
        ),
        "CANVAS related xref must not be treated as exact"
    );
    let e = entity("CANVAS syndrome", "C000726747");
    let mapped = ds
        .mapper()
        .map_entity_with_names(&e, &["CANVAS"])
        .unwrap()
        .require_unique()
        .unwrap();
    assert_eq!(mapped.method, MappingMethod::ExactName);
}

#[tokio::test]
async fn downloader_refuses_to_overwrite_existing_data_before_http() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("existing"), "keep").unwrap();
    assert!(matches!(
        download_data(directory.path()).await,
        Err(Error::InvalidRequest(_))
    ));
    assert_eq!(
        std::fs::read_to_string(directory.path().join("existing")).unwrap(),
        "keep"
    );
}
