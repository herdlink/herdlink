use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::graph::GraphSettings;
use serde_json::Value;
use tower::ServiceExt;

async fn request(app: &Router, path: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().uri(path);
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2_000_000).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned())),
    )
}
async fn demo(app: &Router) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/demo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_000_000).await.unwrap()).unwrap();
    value["token"].as_str().unwrap().into()
}
fn settings(uri: &str) -> GraphSettings {
    GraphSettings {
        uri: uri.into(),
        user: "neo4j".into(),
        password: String::new(),
        database: "neo4j".into(),
    }
}

#[tokio::test]
async fn graph_reads_require_auth_and_validate_bounds_before_connecting() {
    let db = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&db).await.unwrap();
    let app = backend::app_with_graph(db, settings("127.0.0.1:1"));
    assert_eq!(
        request(&app, "/api/graph", None).await.0,
        StatusCode::UNAUTHORIZED
    );
    let token = demo(&app).await;
    for path in [
        "/api/graph?limit=0",
        "/api/graph?limit=401",
        "/api/graph/evidence?id=any&kind=invalid",
        "/api/graph/evidence?id=any&kind=node&offset=-1",
        "/api/graph?complexity=all",
        "/api/graph?complexity=1000000",
    ] {
        assert_eq!(
            request(&app, path, Some(&token)).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    for path in [
        "/api/graph",
        "/api/graph?q=%20%20",
        "/api/graph?result_uid=%20",
        "/api/graph?complexity=expanded",
        "/api/graph?complexity=detailed",
    ] {
        let (status, snapshot) = request(&app, path, Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(snapshot["nodes"].as_array().unwrap().is_empty());
        assert!(snapshot["edges"].as_array().unwrap().is_empty());
    }
    assert_eq!(
        request(&app, "/api/graph?q=Huntington", Some(&token))
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

/// Use an isolated Neo4j instance: NEO4J_TEST_URI=127.0.0.1:17687 cargo test -p backend --test graph -- --ignored
#[tokio::test]
#[ignore = "requires an isolated Neo4j instance (NEO4J_TEST_URI)"]
async fn snapshot_and_evidence_are_read_only_and_preserve_provenance() {
    let uri =
        std::env::var("NEO4J_TEST_URI").expect("set NEO4J_TEST_URI to an isolated test instance");
    let graph = neo4rs::Graph::connect(
        neo4rs::ConfigBuilder::default()
            .uri(uri.clone())
            .user("neo4j")
            .password("")
            .db("neo4j")
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
    // These fixtures reproduce the repository's stored schema; no upstream calls are made.
    graph.run(neo4rs::query("CREATE
      (:GraphNode:Disease {uid:'unrelated',id:'MONDO:UNRELATED',name:'Unconnected disease'}),
      (d:GraphNode:Disease {uid:'disease:MONDO:TEST',id:'MONDO:TEST',name:'Huntington disease'}),
      (e:GraphNode:Entity:DiseaseEntity {uid:'pubtator:@DISEASE_Huntington_Disease',accession:'@DISEASE_Huntington_Disease',name:'Huntington disease'}),
      (g:GraphNode:Entity:Gene {uid:'pubtator:@GENE_HTT',accession:'@GENE_HTT',name:'HTT'}),
      (term:GraphNode:HpoTerm {uid:'hpo:HP:0002071',id:'HP:0002071',name:'Abnormality of movement'}),
      (profile:GraphNode:HpoDiseaseProfile {uid:'profile:test'}),
      (ann:GraphNode:HpoAnnotation {uid:'annotation:positive',excluded:false}),
      (absent:GraphNode:HpoAnnotation {uid:'annotation:absent',excluded:true}),
      (p:GraphNode:Publication {uid:'publication:19894120',pmid:19894120,title:'Recorded Huntington publication'}),
      (p2:GraphNode:Publication {uid:'publication:2',pmid:2,title:'Excluded phenotype citation'}),
      (doc:GraphNode:Document {uid:'document:test'}),
      (rel:GraphNode:ExtractedRelation {uid:'extracted:test',type:'associate',score:'0.91'}),
      (mention:GraphNode:Mention {uid:'mention:test'}),
      (mapping:GraphNode:DiseaseMapping {uid:'mapping:test',method:'ExactMesh'}),
      (summary:GraphNode:RelationSummary {uid:'summary:test',publications:3616}),
      (result:GraphNode:FetchResult {uid:'result:test'}),
      (profile)-[:PROFILE_OF {uid:'profile-of'}]->(d),
      (profile)-[:HAS_PHENOTYPE {uid:'phenotype-edge'}]->(term),
      (profile)-[:EXCLUDES_PHENOTYPE {uid:'excluded-edge'}]->(term),
      (profile)-[:HAS_ANNOTATION {uid:'positive-ann'}]->(ann),
      (profile)-[:HAS_ANNOTATION {uid:'negative-ann'}]->(absent),
      (ann)-[:PHENOTYPE {uid:'positive-term'}]->(term),
      (absent)-[:PHENOTYPE {uid:'negative-term'}]->(term),
      (ann)-[:SUPPORTED_BY {uid:'positive-paper'}]->(p),
      (absent)-[:SUPPORTED_BY {uid:'negative-paper'}]->(p2),
      (p)-[:HAS_DOCUMENT {uid:'doc-edge'}]->(doc),
      (doc)-[:HAS_RELATION {uid:'relation-edge'}]->(rel),
      (rel)-[:HAS_PARTICIPANT {uid:'participant1',role:'role1'}]->(e),
      (rel)-[:HAS_PARTICIPANT {uid:'participant2',role:'role2'}]->(g),
      (doc)-[:HAS_MENTION {uid:'mention-edge'}]->(mention),
      (mention)-[:DENOTES {uid:'denotes-edge'}]->(g),
      (e)-[:MAPPED_VIA {uid:'mapped-via'}]->(mapping),
      (mapping)-[:MAPS_TO {uid:'maps-to'}]->(d),
      (e)-[:PUBTATOR_RELATION {uid:'summary-edge',relation_type:'associate',publications:3616,summary_uid:'summary:test'}]->(g),
      (result)-[:HAS_OBJECT {uid:'result-object'}]->(e),
      (result)-[:HAS_OBJECT {uid:'result-disease'}]->(d)")).await.unwrap();
    for i in 0..24 {
        graph.run(neo4rs::query("CREATE (p:GraphNode:Publication {uid:$id,pmid:$pmid,title:$title}) WITH p MATCH (ann:GraphNode {uid:'annotation:positive'}) CREATE (ann)-[:SUPPORTED_BY {uid:$edge}]->(p)").param("id",format!("publication:test-{i:02}")).param("pmid",100 + i).param("title",format!("Stored evidence {i}")).param("edge",format!("citation-{i}"))).await.unwrap();
    }
    let count = async {
        let mut result = graph.execute(neo4rs::query("MATCH (n) OPTIONAL MATCH (n)-[r]->() RETURN count(DISTINCT n) AS nodes,count(r) AS edges")).await.unwrap();
        let row = result.next().await.unwrap().unwrap();
        (
            row.get::<i64>("nodes").unwrap(),
            row.get::<i64>("edges").unwrap(),
        )
    };
    let before = count.await;
    let db = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&db).await.unwrap();
    let app = backend::app_with_graph(db, settings(&uri));
    let token = demo(&app).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/communities/mondo-test/open")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let community: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_000_000).await.unwrap()).unwrap();
    assert_eq!(
        community["name"], "Huntington disease",
        "Direct community visits resolve the stored disease name"
    );
    let (_, initial) = request(&app, "/api/graph", Some(&token)).await;
    assert!(initial["nodes"].as_array().unwrap().is_empty());
    assert!(initial["edges"].as_array().unwrap().is_empty());
    for path in [
        "/api/graph?q=Huntington",
        "/api/graph?q=Huntington%20disease",
        "/api/graph?q=%40DISEASE_Huntington_Disease",
        "/api/graph?q=MONDO:TEST",
    ] {
        let (status, disease) = request(&app, path, Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "{disease}");
        assert_eq!(disease["nodes"].as_array().unwrap().len(), 1, "{disease}");
        assert_eq!(disease["nodes"][0]["id"], "disease:MONDO:TEST");
        assert_eq!(
            disease["nodes"][0]["community_url"],
            "/community/mondo-test"
        );
        assert!(disease["edges"].as_array().unwrap().is_empty());
    }
    let (_, gene_search) = request(&app, "/api/graph?q=HTT", Some(&token)).await;
    assert!(gene_search["nodes"].as_array().unwrap().is_empty());
    for complexity in ["expanded", "detailed"] {
        let (status, expanded) = request(
            &app,
            &format!("/api/graph?q=Huntington%20disease&complexity={complexity}"),
            Some(&token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{expanded}");
        let nodes = expanded["nodes"].as_array().unwrap();
        assert!(nodes.len() > 1);
        assert!(nodes.len() <= if complexity == "expanded" { 60 } else { 100 });
        assert!(nodes.iter().any(|n| n["id"] == "disease:MONDO:TEST"));
        assert!(!nodes.iter().any(|n| n["id"] == "unrelated"));
        let ids = nodes
            .iter()
            .map(|n| n["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            expanded["edges"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| ids.contains(e["source"].as_str().unwrap())
                    && ids.contains(e["target"].as_str().unwrap()))
        );
    }
    let (status, snapshot) = request(&app, "/api/graph?result_uid=result:test", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{snapshot}");
    let nodes = snapshot["nodes"].as_array().unwrap();
    let edges = snapshot["edges"].as_array().unwrap();
    assert!(nodes.iter().all(|n| {
        !n["labels"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l == "FetchResult" || l == "HpoAnnotation")
    }));
    assert!(nodes.iter().any(|n| n["kind"] == "gene"));
    let diseases: Vec<_> = nodes.iter().filter(|n| n["kind"] == "disease").collect();
    assert_eq!(diseases.len(), 2);
    assert_eq!(diseases[0]["community_url"], diseases[1]["community_url"]);
    assert!(
        edges
            .iter()
            .any(|e| e["kind"] == "HAS_PHENOTYPE" && e["source"] == "disease:MONDO:TEST")
    );
    assert!(edges.iter().any(|e| e["kind"] == "EXCLUDES_PHENOTYPE"));
    assert!(edges.iter().any(|e| e["kind"] == "ANNOTATION_CITATION"
        && e["source"] == "publication:19894120"
        && e["target"] == "disease:MONDO:TEST"));
    assert!(
        edges
            .iter()
            .any(|e| e["kind"] == "EXTRACTED_RELATION" && e["context"]["score"] == "0.91")
    );
    assert!(
        edges
            .iter()
            .any(|e| e["kind"] == "MENTIONS" && e["label"] == "mentions")
    );
    assert!(
        edges
            .iter()
            .any(|e| e["reported_papers"] == 3616 && e["evidence_kind"] == "summary")
    );
    let (status, papers) = request(
        &app,
        "/api/graph/evidence?id=phenotype-edge&kind=phenotype",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{papers}");
    assert_eq!(papers["papers"].as_array().unwrap().len(), 20);
    assert_eq!(papers["has_more"], true);
    assert!(
        papers["papers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["id"] != "publication:2")
    );
    let (_, next) = request(
        &app,
        "/api/graph/evidence?id=phenotype-edge&kind=phenotype&offset=20",
        Some(&token),
    )
    .await;
    assert_eq!(next["papers"].as_array().unwrap().len(), 5);
    assert_eq!(next["has_more"], false);
    let (_, excluded) = request(
        &app,
        "/api/graph/evidence?id=excluded-edge&kind=phenotype",
        Some(&token),
    )
    .await;
    assert_eq!(excluded["papers"][0]["id"], "publication:2");
    assert_eq!(excluded["papers"].as_array().unwrap().len(), 1);
    let (status, extracted) = request(
        &app,
        "/api/graph/evidence?id=extracted:test&kind=extracted",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(extracted["papers"][0]["id"], "publication:19894120");
    let (status, associated) = request(
        &app,
        "/api/graph/evidence?id=pubtator:%40GENE_HTT&kind=node",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{associated}");
    assert_eq!(associated["papers"].as_array().unwrap().len(), 1);
    let (_, summary) = request(
        &app,
        "/api/graph/evidence?id=summary:test&kind=summary",
        Some(&token),
    )
    .await;
    assert!(summary["papers"].as_array().unwrap().is_empty());
    assert!(summary["note"].as_str().unwrap().contains("count"));
    let (_, bounded) = request(
        &app,
        "/api/graph?result_uid=result:test&limit=2",
        Some(&token),
    )
    .await;
    assert_eq!(bounded["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(bounded["truncated"], true);
    let (_, missing) = request(&app, "/api/graph?q=NoSuchTopic", Some(&token)).await;
    assert!(missing["nodes"].as_array().unwrap().is_empty());
    let (status, scoped) = request(&app, "/api/graph?result_uid=result:test", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        scoped["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["id"] == "pubtator:@DISEASE_Huntington_Disease")
    );
    let mut result = graph.execute(neo4rs::query("MATCH (n) OPTIONAL MATCH (n)-[r]->() RETURN count(DISTINCT n) AS nodes,count(r) AS edges")).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(
        before,
        (
            row.get::<i64>("nodes").unwrap(),
            row.get::<i64>("edges").unwrap()
        ),
        "Graph endpoints must not mutate Neo4j"
    );
}
