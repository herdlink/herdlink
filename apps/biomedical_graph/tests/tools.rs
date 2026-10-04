use biomedical_graph::{CachePolicy, CachedHpo, CachedPubTator, Store, tools::GraphTools};
use openai::{FunctionCall, InputItem};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

async fn uids(store: &Store) -> BTreeSet<String> {
    let mut stream = store
        .graph()
        .execute(neo4rs::query("MATCH (n:GraphNode) RETURN n.uid AS uid"))
        .await
        .unwrap();
    let mut ids = BTreeSet::new();
    while let Some(row) = stream.next().await.unwrap() {
        ids.insert(row.get("uid").unwrap());
    }
    ids
}

#[tokio::test]
#[ignore = "requires Neo4j; run cargo test -p biomedical_graph --test tools -- --ignored"]
async fn all_tools_dispatch_and_replay_offline_from_persistent_cache() {
    let config = neo4rs::ConfigBuilder::default()
        .uri(std::env::var("NEO4J_URI").unwrap_or_else(|_| "127.0.0.1:7687".into()))
        .user(std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into()))
        .password(std::env::var("NEO4J_PASSWORD").unwrap_or_default())
        .db(std::env::var("NEO4J_DATABASE").unwrap_or_else(|_| "neo4j".into()))
        .build()
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let store = Store::connect_with_cache_dir(config, directory.path())
        .await
        .unwrap();
    let baseline = uids(&store).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let received = count.clone();
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut buffer = [0; 4096];
                let n = socket.read(&mut buffer).await.unwrap();
                assert_ne!(n, 0);
                request.extend_from_slice(&buffer[..n]);
                if request.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            received.fetch_add(1, Ordering::SeqCst);
            let request = String::from_utf8(request).unwrap();
            let body = if request.contains("lookup/details") {
                include_str!("../../pubtator3/tests/fixtures/mesh-als.json")
            } else if request.contains("/relations?") {
                include_str!("fixtures/pubtator-relations.json")
            } else if request.contains("/search/") {
                include_str!("../../pubtator3/tests/fixtures/search.json")
            } else if request.contains("publications/export/pubtator?") {
                "19894120|t|Example title\n19894120|a|Example abstract\n"
            } else if request.contains("/biocxml?") {
                "<collection><document><id>19894120</id></document></collection>"
            } else if request.contains("publications/") {
                include_str!("fixtures/pubtator-document.json")
            } else {
                include_str!("fixtures/pubtator-entities.json")
            };
            socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
        }
    });
    let upstream = pubtator3::Client::builder()
        .base_url(format!("http://{address}/"))
        .mesh_base_url(format!("http://{address}/mesh/"))
        .timeout(Duration::from_secs(2))
        .request_interval(Duration::ZERO)
        .build()
        .unwrap();
    // Add two explicitly synthetic profiles only in this test. A one-disease
    // corpus gives every term zero information content and cannot rank similarity.
    let annotations = format!(
        "{}OMIM:100001\tSynthetic test disease A\t\tHP:0001250\t\tIEA\t\t\t\t\tP\t\nOMIM:100002\tSynthetic test disease B\t\tHP:0000496\t\tIEA\t\t\t\t\tP\t\n",
        include_str!("fixtures/phenotype.hpoa")
    );
    let dataset = pubtator3_hpo::Dataset::from_readers(
        std::io::Cursor::new(include_str!("fixtures/hp.obo")),
        annotations.as_bytes(),
        include_str!("fixtures/mondo.json").as_bytes(),
    )
    .unwrap();
    let dataset = Arc::new(dataset);
    let make_tools = || {
        GraphTools::new(CachedPubTator::new(
            upstream.clone(),
            store.clone(),
            CachePolicy::default(),
        ))
        .with_hpo(CachedHpo::new(
            pubtator3_hpo::Client::new(upstream.clone(), dataset.clone()),
            store.clone(),
            CachePolicy::default(),
        ))
    };
    let tools = make_tools();
    assert_eq!(tools.definitions().len(), 17);
    assert_eq!(
        GraphTools::new(CachedPubTator::new(
            upstream.clone(),
            store.clone(),
            CachePolicy::default()
        ))
        .definitions()
        .len(),
        11
    );
    let query = json!({"query":"Huntington disease", "selected":"@DISEASE_Huntington_Disease"});
    let mut calls: Vec<(&str, Value)> = vec![
        (
            "pubtator_autocomplete",
            json!({"query":"Huntington disease", "concept":"disease", "limit":2}),
        ),
        (
            "pubtator_relations",
            json!({"entity":"@DISEASE_Huntington_Disease", "target_type":"gene", "relation_type":"associate", "limit":2}),
        ),
        ("pubtator_search", json!({"query":"HTT", "page":1})),
        (
            "pubtator_annotations",
            json!({"pmids":["19894120"], "scope":"abstract"}),
        ),
        (
            "pubtator_annotations_batched",
            json!({"pmids":vec!["19894120"; 101], "scope":"full_text"}),
        ),
        ("pubtator_pmc_annotations", json!({"pmcids":["PMC6142073"]})),
        ("pubtator_mesh_synonyms", json!({"descriptor":"D000690"})),
        (
            "pubtator_synonyms",
            json!({"entity":{"_id":"@DISEASE_Amyotrophic_Lateral_Sclerosis", "name":"ALS", "biotype":"disease", "db":"ncbi_mesh", "db_id":"D000690"}}),
        ),
        (
            "pubtator_export",
            json!({"pmids":["19894120"], "format":"biocjson", "scope":"abstract"}),
        ),
        (
            "pubtator_pmc_export",
            json!({"pmcids":["PMC6142073"], "format":"biocjson"}),
        ),
        ("hpo_resolve_disease", query.clone()),
        ("hpo_disease_phenotypes", query),
        ("hpo_profile", json!({"id":"MONDO:0007739"})),
        (
            "hpo_similar_diseases",
            json!({"query":"Huntington disease", "selected":"@DISEASE_Huntington_Disease", "limit":2, "min_score":0.0, "min_phenotypes":1}),
        ),
        (
            "hpo_supporting_papers",
            json!({"source":"@DISEASE_Huntington_Disease", "target":"@DISEASE_Parkinson_Disease", "page":1}),
        ),
    ];
    calls.push(("pubtator_relation_papers", json!({"source":"@DISEASE_Huntington_Disease","target":"@GENE_HTT","relation_type":"associate","page":1})));
    let mut values = Vec::new();
    for (name, args) in &calls {
        let (value, graph) = tools
            .execute_with_graph(name, &args.to_string())
            .await
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let persisted = uids(&store).await;
        assert!(
            graph.nodes.keys().all(|uid| persisted.contains(uid)),
            "{name}: live graph only contains persisted tool objects"
        );
        if *name == "hpo_similar_diseases" {
            assert_eq!(
                graph
                    .nodes
                    .values()
                    .filter(|node| node.labels.iter().any(|label| label == "SimilarityResult"))
                    .count(),
                2
            );
        }
        values.push(value);
    }
    assert_eq!(values[0].as_array().unwrap().len(), 2);
    assert_eq!(values[1][0]["type"], "associate");
    assert_eq!(
        values[4].as_array().unwrap().len(),
        2,
        "101 IDs produce two cached batches"
    );
    assert_eq!(values[11]["mapping"]["method"], "ExactMesh");
    assert_eq!(values[13]["matches"].as_array().unwrap().len(), 2);
    let profile = &values[12];
    let reverse = json!({"disease":{
        "disease_id":profile["id"], "name":profile["name"], "mondo":profile["mondo"],
        "annotation_ids":profile["annotation_ids"], "mesh_ids":profile["mesh_ids"],
        "score":1.0, "phenotype_count":profile["phenotypes"].as_array().unwrap().len(),
        "shared_phenotypes":profile["phenotypes"], "source_only_phenotypes":[], "conflicting_phenotypes":[],
    }});
    values.push(
        tools
            .execute("hpo_pubtator_entities", &reverse.to_string())
            .await
            .unwrap(),
    );
    calls.push(("hpo_pubtator_entities", reverse));
    for format in ["pubtator", "biocxml"] {
        for (name, ids) in [
            (
                "pubtator_export",
                json!({"pmids":["19894120"], "scope":"abstract"}),
            ),
            ("pubtator_pmc_export", json!({"pmcids":["PMC6142073"]})),
        ] {
            if name == "pubtator_pmc_export" && format == "pubtator" {
                continue;
            }
            let mut args = ids;
            args["format"] = json!(format);
            let result = tools.execute(name, &args.to_string()).await.unwrap();
            assert!(!result.as_str().unwrap().is_empty());
            calls.push((name, args));
            values.push(result);
        }
    }
    // No server remains: fresh wrappers must serve every result from Neo4j/disk.
    server.abort();
    let fetched = count.load(Ordering::SeqCst);
    let fresh = make_tools();
    for ((name, args), expected) in calls.iter().zip(values) {
        assert_eq!(
            fresh.execute(name, &args.to_string()).await.unwrap(),
            expected,
            "{name}"
        );
    }
    assert_eq!(count.load(Ordering::SeqCst), fetched);
    assert!(
        fresh
            .execute("pubtator_annotations", r#"{"pmids":[],"scope":"abstract"}"#)
            .await
            .is_err()
    );
    assert!(fresh.execute("hpo_similar_diseases", r#"{"query":"Huntington disease","selected":null,"limit":1,"min_score":2,"min_phenotypes":1}"#).await.is_err());
    let call = FunctionCall {
        id: "fc_test".into(),
        call_id: "call_test".into(),
        name: "unknown".into(),
        arguments: "{}".into(),
    };
    let InputItem::FunctionCallOutput { call_id, output } = fresh.call(&call).await else {
        panic!("expected tool output")
    };
    assert_eq!(call_id, "call_test");
    assert!(
        serde_json::from_str::<Value>(&output).unwrap()["error"]
            .as_str()
            .unwrap()
            .contains("unknown")
    );
    let created: Vec<_> = uids(&store).await.difference(&baseline).cloned().collect();
    store
        .graph()
        .run(
            neo4rs::query("UNWIND $ids AS uid MATCH (n:GraphNode {uid:uid}) DETACH DELETE n")
                .param("ids", created),
        )
        .await
        .unwrap();
}
