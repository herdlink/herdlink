use biomedical_graph::*;
use pubtator3::{AutocompleteRequest, Concept, Pmid, TextScope};
use serde_json::json;
use std::{
    collections::BTreeSet,
    num::NonZeroU32,
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

/// Requires the local Neo4j instance. No requests leave localhost.
#[tokio::test]
#[ignore = "requires Neo4j; run cargo test -p biomedical_graph --test neo4j -- --ignored"]
async fn persistent_cache_transactions_ttl_and_extensions() {
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
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let task = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            loop {
                let mut buffer = [0; 4096];
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..n]);
                if bytes.windows(4).any(|v| v == b"\r\n\r\n") {
                    break;
                }
            }
            count.fetch_add(1, Ordering::SeqCst);
            let request = String::from_utf8(bytes).unwrap();
            let fail = request.contains("query=error");
            let body = if fail {
                "upstream unavailable".to_owned()
            } else if request.contains("publications/") {
                let mut value: serde_json::Value =
                    serde_json::from_str(include_str!("fixtures/pubtator-document.json")).unwrap();
                if request.contains("full=true") {
                    value["PubTator3"][0]["passages"].as_array_mut().unwrap().push(json!({"infons":{"section_type":"INTRO"},"text":"HIDDEN_ARTICLE_SENTINEL","offset":10000,"sentences":[{"offset":10000,"text":"HIDDEN_SENTENCE_SENTINEL"}]}));
                }
                value.to_string()
            } else {
                include_str!("fixtures/pubtator-entities.json").to_owned()
            };
            let status = if fail { 500 } else { 200 };
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let upstream = pubtator3::Client::builder()
        .base_url(format!("http://{address}/"))
        .request_interval(Duration::ZERO)
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let client = CachedPubTator::new(upstream.clone(), store.clone(), CachePolicy::default());
    let request = AutocompleteRequest::new("Huntington disease")
        .concept(Concept::Disease)
        .limit(NonZeroU32::new(2).unwrap());
    let (first, second) =
        tokio::join!(client.autocomplete(&request), client.autocomplete(&request));
    assert_eq!(first.unwrap().len(), 2);
    assert_eq!(second.unwrap().len(), 2);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "clones coalesce concurrent misses"
    );
    let fresh = CachedPubTator::new(upstream.clone(), store.clone(), CachePolicy::default());
    assert_eq!(fresh.autocomplete(&request).await.unwrap().len(), 2);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "cache persists across wrapper reconstruction"
    );
    // Full-text/abstract and pagination/filter inputs have distinct keys.
    let pmid = Pmid::new(19894120).unwrap();
    client
        .annotations(&[pmid], TextScope::Abstract)
        .await
        .unwrap();
    let full_text = client
        .annotations(&[pmid], TextScope::FullText)
        .await
        .unwrap();
    assert_eq!(
        full_text[0].passages.last().unwrap().text.as_deref(),
        Some("HIDDEN_ARTICLE_SENTINEL")
    );
    let cached_full_text = client
        .annotations(&[pmid], TextScope::FullText)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&full_text).unwrap(),
        serde_json::to_value(cached_full_text).unwrap()
    );
    let mut stream=store.graph().execute(neo4rs::query("MATCH (n:GraphNode) WHERE any(key IN keys(n) WHERE toString(n[key]) CONTAINS 'HIDDEN_ARTICLE_SENTINEL' OR toString(n[key]) CONTAINS 'HIDDEN_SENTENCE_SENTINEL') RETURN count(n) AS count")).await.unwrap();
    assert_eq!(
        stream
            .next()
            .await
            .unwrap()
            .unwrap()
            .get::<i64>("count")
            .unwrap(),
        0
    );
    let mut stream=store.graph().execute(neo4rs::query("MATCH (n:GraphNode) WHERE n.payload_json IS NOT NULL OR n:Passage OR n:Sentence RETURN count(n) AS count")).await.unwrap();
    assert_eq!(
        stream
            .next()
            .await
            .unwrap()
            .unwrap()
            .get::<i64>("count")
            .unwrap(),
        0
    );
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    let dataset =
        pubtator3_hpo::Dataset::from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))
            .unwrap();
    let hpo = CachedHpo::new(
        pubtator3_hpo::Client::new(upstream.clone(), dataset),
        store.clone(),
        CachePolicy::default(),
    );
    let mapped = hpo
        .disease_phenotypes("Huntington disease", None)
        .await
        .unwrap();
    assert_eq!(
        mapped.mapping.method,
        pubtator3_hpo::MappingMethod::ExactMesh
    );
    hpo.disease_phenotypes("Huntington disease", None)
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    let failed = AutocompleteRequest::new("error");
    assert!(client.autocomplete(&failed).await.is_err());
    assert!(client.autocomplete(&failed).await.is_err());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        6,
        "upstream failures are never cached"
    );
    let key = client.cache_key(
        "autocomplete",
        &(&request.query, request.concept, request.limit),
    );
    store
        .graph()
        .run(
            neo4rs::query("MATCH (c:GraphNode {uid:$key}) SET c.expires_at=0")
                .param("key", key.clone()),
        )
        .await
        .unwrap();
    client.autocomplete(&request).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 7, "expired entries refresh");
    let mut result = store
        .graph()
        .execute(
            neo4rs::query(
                "MATCH (c:GraphNode {uid:$key})-[r:CACHED_RESULT]->() RETURN count(r) AS count",
            )
            .param("key", key.clone()),
        )
        .await
        .unwrap();
    assert_eq!(
        result
            .next()
            .await
            .unwrap()
            .unwrap()
            .get::<i64>("count")
            .unwrap(),
        1
    );
    store.invalidate(&key).await.unwrap();
    client.autocomplete(&request).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 8);

    let bypass_request = AutocompleteRequest::new("zero ttl");
    client.autocomplete(&bypass_request).await.unwrap();
    let bypass = CachedPubTator::new(
        upstream.clone(),
        store.clone(),
        CachePolicy {
            ttl: Duration::ZERO,
        },
    );
    bypass.autocomplete(&bypass_request).await.unwrap();
    bypass.autocomplete(&bypass_request).await.unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        11,
        "zero TTL bypasses even an existing valid entry"
    );

    let mut extension = GraphBatch::default();
    let trial = extension.node(
        format!("test-trial:{address}"),
        "ClinicalTrial",
        Properties::from([("phase".into(), json!(2))]),
    );
    let disease = extension.node("disease:MONDO:0007739", "Disease", Properties::new());
    extension.edge(&trial, "STUDIES", &disease, "", Properties::new());
    store.upsert(&extension).await.unwrap();
    let before = uids(&store).await;
    store.upsert(&extension).await.unwrap();
    assert_eq!(before, uids(&store).await);
    let mut invalid = extension.clone();
    invalid.node("invalid:test", "Bad`Cypher", Properties::new());
    assert!(store.upsert(&invalid).await.is_err());
    assert!(!uids(&store).await.contains("invalid:test"));
    // A server-side property error after node creation must roll back the whole batch.
    let rollback_uid = format!("test-rollback:{address}");
    let mut bad = GraphBatch::default();
    bad.node(
        &rollback_uid,
        "FutureType",
        Properties::from([("".into(), json!("invalid property key"))]),
    );
    assert!(store.upsert(&bad).await.is_err());
    assert!(!uids(&store).await.contains(&rollback_uid));
    task.abort();
    assert_eq!(
        fresh.autocomplete(&request).await.unwrap().len(),
        2,
        "cached responses work with upstream offline"
    );
    store.prune_cache().await.unwrap();
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
