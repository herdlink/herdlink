use axum::{
    Json, Router,
    body::{Body, Bytes, to_bytes},
    extract::State,
    http::{Request, Response, StatusCode},
    routing::{get, post},
};
use backend::{chat::ChatService, graph::GraphSettings};
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{Notify, mpsc};
use tower::ServiceExt;

fn settings() -> GraphSettings {
    GraphSettings {
        uri: "127.0.0.1:1".into(),
        user: "neo4j".into(),
        password: String::new(),
        database: "neo4j".into(),
    }
}
fn event(value: Value) -> String {
    format!("data: {value}\n\n")
}
fn message(text: &str) -> Value {
    json!({"type":"message","id":"msg","role":"assistant","content":[{"type":"output_text","text":text}]})
}
fn call(id: &str, name: &str, args: Value) -> Value {
    json!({"type":"function_call","id":format!("fc_{id}"),"call_id":id,"name":name,"arguments":args.to_string()})
}
fn reply(text: &str, calls: Vec<Value>, pause: Option<Arc<Notify>>) -> Response<Body> {
    let delta = event(
        json!({"type":"response.output_text.delta","item_id":"msg","output_index":0,"content_index":0,"delta":text}),
    );
    let mut output = calls;
    if !text.is_empty() {
        output.push(message(text));
    }
    let complete = event(
        json!({"type":"response.completed","response":{"id":"resp","status":"completed","store":false,"output":output}}),
    );
    let has_text = !text.is_empty();
    let body = async_stream::stream! {
        if has_text { yield Ok::<_,Infallible>(Bytes::from(delta)); }
        if let Some(pause) = pause { pause.notified().await; }
        yield Ok(Bytes::from(complete));
    };
    Response::builder()
        .header("content-type", "text/event-stream")
        .body(Body::from_stream(body))
        .unwrap()
}
#[derive(Clone)]
struct Mock {
    replies: Arc<Mutex<VecDeque<Response<Body>>>>,
    requests: mpsc::UnboundedSender<Value>,
}
async fn model(State(state): State<Mock>, Json(body): Json<Value>) -> Response<Body> {
    let _ = state.requests.send(body);
    state
        .replies
        .lock()
        .unwrap()
        .pop_front()
        .expect("unexpected model call")
}
async fn model_server(
    replies: Vec<Response<Body>>,
) -> (
    openai::Client,
    mpsc::UnboundedReceiver<Value>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = openai::Client::builder("fixture-key")
        .base_url(format!("http://{}/v1", listener.local_addr().unwrap()))
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let router = Router::new()
        .route("/v1/responses", post(model))
        .with_state(Mock {
            replies: Arc::new(Mutex::new(replies.into())),
            requests: tx,
        });
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (client, rx, task)
}
async fn json_request(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    value: Value,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(value.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 4_000_000).await.unwrap();
    (
        status,
        serde_json::from_slice(&body)
            .unwrap_or_else(|_| Value::String(String::from_utf8(body.to_vec()).unwrap())),
    )
}
async fn demo(app: &Router) -> String {
    let (_, v) = json_request(app, "POST", "/api/auth/demo", None, json!({})).await;
    v["token"].as_str().unwrap().into()
}
async fn stream_request(app: &Router, token: &str, value: Value) -> Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}
fn frames(body: &str, kind: &str) -> Vec<Value> {
    body.split("\n\n")
        .filter(|frame| frame.lines().any(|line| line == format!("event: {kind}")))
        .filter_map(|frame| {
            frame
                .lines()
                .find_map(|line| line.strip_prefix("data: "))
                .map(|data| serde_json::from_str(data).unwrap())
        })
        .collect()
}
#[tokio::test]
async fn chat_complexity_reveals_saved_sources_without_changing_messages_or_owner_scope() {
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app_with_graph(pool.clone(), settings());
    let token = demo(&app).await;
    let (_, user) = json_request(&app, "GET", "/api/me", Some(&token), json!(null)).await;
    let owner = user["id"].as_str().unwrap().parse::<uuid::Uuid>().unwrap();
    let id = uuid::Uuid::new_v4();
    let nodes = (0..80).map(|i| json!({
        "id":format!("node:{i}"),"label":format!("Source {i}"),"kind":if i == 0 {"disease"} else {"gene"},
        "labels":[],"description":null,"links":[],"community_url":null,"reasons":["Stored tool result"]
    })).collect::<Vec<_>>();
    let edges = (1..80).map(|i| json!({
        "id":format!("edge:{i}"),"source":"node:0","target":format!("node:{i}"),"label":"associate",
        "kind":"PUBTATOR_RELATION","evidence_id":format!("summary:{i}"),"evidence_kind":"summary",
        "reported_papers":2,"context":{"added_by":"Find genes"}
    })).collect::<Vec<_>>();
    let stored = json!({"nodes":nodes,"edges":edges,"query":"Source 0","result_uid":"","truncated":false,"generated_at":"2026-10-04T00:00:00Z"}).to_string();
    let messages = json!([{"role":"user","content":"Find genes"},{"role":"assistant","content":"Here are the stored associations."}]).to_string();
    sqlx::query("INSERT INTO graph_chats(id,user_id,title,messages_json,graph_json) VALUES(?1,?2,'Complexity',?3,?4)")
        .bind(id).bind(owner).bind(&messages).bind(&stored).execute(&pool).await.unwrap();
    for (complexity, count) in [
        ("focused", 30),
        ("expanded", 60),
        ("detailed", 80),
        ("focused", 30),
    ] {
        let (status, conversation) = json_request(
            &app,
            "GET",
            &format!("/api/chats/{id}?complexity={complexity}"),
            Some(&token),
            json!(null),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{conversation}");
        assert_eq!(
            conversation["graph"]["nodes"].as_array().unwrap().len(),
            count
        );
        assert_eq!(
            conversation["messages"],
            serde_json::from_str::<Value>(&messages).unwrap()
        );
        assert!(
            conversation["graph"]["edges"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["context"]["added_by"] == "Find genes")
        );
    }
    let (_, default) = json_request(
        &app,
        "GET",
        &format!("/api/chats/{id}"),
        Some(&token),
        json!(null),
    )
    .await;
    assert_eq!(default["graph"]["nodes"].as_array().unwrap().len(), 30);
    let persisted: String = sqlx::query_scalar("SELECT graph_json FROM graph_chats WHERE id=?1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        persisted, stored,
        "View changes must not discard saved detail"
    );
    assert_eq!(
        json_request(
            &app,
            "GET",
            &format!("/api/chats/{id}?complexity=all"),
            Some(&token),
            json!(null)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_, other) = json_request(&app,"POST","/api/auth/register",None,json!({"email":"detail-private@example.test","username":"detail_private","password":"strong-password-123"})).await;
    assert_eq!(
        json_request(
            &app,
            "GET",
            &format!("/api/chats/{id}?complexity=detailed"),
            Some(other["token"].as_str().unwrap()),
            json!(null)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn chat_streams_before_completion_without_tools_and_replays_private_history() {
    let pause = Arc::new(Notify::new());
    let (client, mut requests, server) = model_server(vec![
        reply("Hello 🌱", vec![], Some(pause.clone())),
        reply("You said hello.", vec![], None),
    ])
    .await;
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let service = Arc::new(ChatService::from_env(settings()).with_client(client, "fixture-model"));
    let app = backend::app_with_chat(pool.clone(), settings(), service);
    let token = demo(&app).await;
    let response = stream_request(&app, &token, json!({"message":"hello"})).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut body = response.into_body().into_data_stream();
    let mut received = String::new();
    while !received.contains("Hello 🌱") {
        let chunk = tokio::time::timeout(Duration::from_secs(2), body.next())
            .await
            .expect("text must arrive before model completion")
            .unwrap()
            .unwrap();
        received.push_str(std::str::from_utf8(&chunk).unwrap());
    }
    assert!(!received.contains("event: done"));
    let id = frames(&received, "conversation")[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    // A second request cannot race this unfinished conversation.
    let conflict =
        stream_request(&app, &token, json!({"conversation_id":id,"message":"race"})).await;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    pause.notify_one();
    while let Some(chunk) = body.next().await {
        received.push_str(std::str::from_utf8(&chunk.unwrap()).unwrap());
    }
    assert_eq!(frames(&received, "done").len(), 1);
    assert!(frames(&received, "graph").is_empty());
    assert!(frames(&received, "tool").is_empty());
    let request = requests.recv().await.unwrap();
    assert_eq!(request["tool_choice"], "auto");
    assert_eq!(request["store"], false);
    assert_eq!(request["stream"], true);
    let (status, chat) = json_request(
        &app,
        "GET",
        &format!("/api/chats/{id}"),
        Some(&token),
        json!(null),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(chat["messages"][1]["content"], "Hello 🌱");
    assert!(chat["graph"]["nodes"].as_array().unwrap().is_empty());
    let (_, list) = json_request(&app, "GET", "/api/chats", Some(&token), json!(null)).await;
    assert_eq!(list[0]["id"], id);
    let response = stream_request(
        &app,
        &token,
        json!({"conversation_id":id,"message":"What did I say?"}),
    )
    .await;
    let text = String::from_utf8(
        to_bytes(response.into_body(), 1_000_000)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(frames(&text, "done").len(), 1);
    let followup = requests.recv().await.unwrap();
    assert!(followup["input"].to_string().contains("Hello 🌱"));
    assert!(followup["input"].to_string().contains("What did I say?"));
    let (_,other)=json_request(&app,"POST","/api/auth/register",None,json!({"email":"private@example.test","username":"private_user","password":"strong-password-123"})).await;
    let token2 = other["token"].as_str().unwrap();
    assert_eq!(
        json_request(
            &app,
            "GET",
            &format!("/api/chats/{id}"),
            Some(token2),
            json!(null)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        stream_request(
            &app,
            token2,
            json!({"conversation_id":id,"message":"steal"})
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    for value in [
        json!({"message":""}),
        json!({"message":"x".repeat(4001)}),
        json!({"message":"hello","disease":""}),
    ] {
        assert_eq!(
            stream_request(&app, &token, value).await.status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        json_request(&app, "GET", "/api/chats", None, json!(null))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    server.abort();
}

#[tokio::test]
async fn disconnect_releases_conversation_and_failed_model_stream_reports_error() {
    let pause = Arc::new(Notify::new());
    let failed=Response::builder().header("content-type","text/event-stream").body(Body::from(event(json!({"type":"response.output_text.delta","item_id":"msg","output_index":0,"content_index":0,"delta":"partial"})))).unwrap();
    let (client, _, server) =
        model_server(vec![reply("starting", vec![], Some(pause.clone())), failed]).await;
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app_with_chat(
        pool,
        settings(),
        Arc::new(ChatService::from_env(settings()).with_client(client, "fixture-model")),
    );
    let token = demo(&app).await;
    let response = stream_request(&app, &token, json!({"message":"hello"})).await;
    let mut stream = response.into_body().into_data_stream();
    let first = stream.next().await.unwrap().unwrap();
    let id = frames(std::str::from_utf8(&first).unwrap(), "conversation")[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    // Let the first model request start before cancelling its unfinished stream.
    loop {
        let chunk = stream.next().await.unwrap().unwrap();
        if std::str::from_utf8(&chunk).unwrap().contains("starting") {
            break;
        }
    }
    drop(stream);
    pause.notify_one();
    let response = stream_request(
        &app,
        &token,
        json!({"conversation_id":id,"message":"continue"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let text = String::from_utf8(
        to_bytes(response.into_body(), 1_000_000)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(!frames(&text, "error").is_empty());
    assert!(frames(&text, "done").is_empty());
    server.abort();
}

#[tokio::test]
#[ignore = "requires isolated Neo4j (NEO4J_TEST_URI)"]
async fn streamed_tools_persist_bounded_graph_and_relation_literature_once() {
    let uri = std::env::var("NEO4J_TEST_URI").expect("isolated Neo4j URI");
    let (client,mut requests,server)=model_server(vec![
        reply("Resolving the disease.",vec![call("resolve","pubtator_autocomplete",json!({"query":"Huntington disease","concept":"disease","limit":1}))],None),
        reply("Finding genes.",vec![call("genes","pubtator_relations",json!({"entity":"@DISEASE_Huntington_Disease","target_type":"gene","relation_type":"associate","limit":2}))],None),
        reply("Reading the association literature.",vec![call("papers","pubtator_relation_papers",json!({"source":"@DISEASE_Huntington_Disease","target":"@GENE_HTT","relation_type":"associate","page":1}))],None),
        reply("HTT is associated with Huntington disease; these papers explain the reported link.",vec![],None),
    ]).await;
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let upstream=Router::new().route("/{*path}",get(move |request: axum::extract::Request| { let tx=tx.clone(); async move {
        let path=request.uri().to_string();tx.send(path.clone()).unwrap();
        let body=if path.contains("relations?") { include_str!("../../biomedical_graph/tests/fixtures/pubtator-relations.json").to_string() }
        else if path.contains("search/") { json!({"results":[{"pmid":19894120,"title":"Huntington association fixture"}],"count":1,"page_size":1,"current":1,"total_pages":1}).to_string() }
        else { include_str!("../../biomedical_graph/tests/fixtures/pubtator-entities.json").to_string() };
        ([ ("content-type","application/json") ],body)
    }}));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let upstream_task = tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    let graph_settings = GraphSettings { uri, ..settings() };
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app_with_chat(
        pool.clone(),
        graph_settings.clone(),
        Arc::new(
            ChatService::from_env(graph_settings)
                .with_client(client, "fixture-model")
                .with_pubtator_url(url),
        ),
    );
    let token = demo(&app).await;
    let response = stream_request(
        &app,
        &token,
        json!({"message":"Find similar diseases through genes and papers"}),
    )
    .await;
    let text = String::from_utf8(
        to_bytes(response.into_body(), 4_000_000)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(frames(&text, "error").is_empty(), "{text}");
    assert_eq!(frames(&text, "done").len(), 1);
    let graphs = frames(&text, "graph");
    assert_eq!(graphs.len(), 3);
    let final_graph = &graphs[2]["snapshot"];
    assert!(final_graph["nodes"].as_array().unwrap().len() <= 180);
    assert!(
        final_graph["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["id"] == "pubtator:@GENE_HTT")
    );
    assert!(
        final_graph["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["kind"] == "RELATION_CITATION"
                && edge["source"] == "publication:19894120")
    );
    let relation = final_graph["edges"]
        .as_array()
        .unwrap()
        .iter()
        .find(|edge| edge["kind"] == "RELATION_PAPERS")
        .unwrap();
    let (_, evidence) = json_request(
        &app,
        "GET",
        &format!(
            "/api/graph/evidence?id={}&kind=search",
            relation["evidence_id"].as_str().unwrap()
        ),
        Some(&token),
        json!(null),
    )
    .await;
    assert_eq!(evidence["papers"][0]["id"], "publication:19894120");
    let id = frames(&text, "conversation")[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let (_, stored) = json_request(
        &app,
        "GET",
        &format!("/api/chats/{id}"),
        Some(&token),
        json!(null),
    )
    .await;
    assert_eq!(stored["graph"], *final_graph);
    assert!(
        stored["graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|node| !node["reasons"].as_array().unwrap().is_empty())
    );
    let mut upstream_calls = Vec::new();
    while let Ok(path) = rx.try_recv() {
        upstream_calls.push(path);
    }
    assert_eq!(upstream_calls.len(), 3, "tools execute once");
    let mut model_requests = Vec::new();
    while let Ok(request) = requests.try_recv() {
        model_requests.push(request);
    }
    assert_eq!(model_requests.len(), 4);
    let last = &model_requests[3]["input"];
    assert!(last.to_string().contains("function_call_output"));
    assert!(last.to_string().contains("pubtator_relation_papers"));
    server.abort();
    upstream_task.abort();
}

#[tokio::test]
#[ignore = "requires isolated Neo4j (NEO4J_TEST_URI)"]
async fn similarity_citations_are_scoped_to_the_scored_snapshot() {
    let uri = std::env::var("NEO4J_TEST_URI").expect("isolated Neo4j URI");
    let graph = neo4rs::Graph::connect(
        neo4rs::ConfigBuilder::default()
            .uri(&uri)
            .user("neo4j")
            .password("")
            .db("neo4j")
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
    let prefix = format!("snapshot-scope:{}", uuid::Uuid::new_v4());
    graph.run(neo4rs::query(&"CREATE
      (s:GraphNode:SimilarityResult {uid:'snapshot-scope:similarity',snapshot:'snapshot-scope:current'}),
      (source:GraphNode:Disease {uid:'snapshot-scope:source'}), (target:GraphNode:Disease {uid:'snapshot-scope:target'}),
      (current:GraphNode:DatasetSnapshot {uid:'snapshot-scope:current',fingerprint:'snapshot-scope:current'}),
      (old:GraphNode:DatasetSnapshot {uid:'snapshot-scope:old',fingerprint:'snapshot-scope:old'}),
      (sp:GraphNode:HpoDiseaseProfile {uid:'snapshot-scope:sp'}), (tp:GraphNode:HpoDiseaseProfile {uid:'snapshot-scope:tp'}),
      (op:GraphNode:HpoDiseaseProfile {uid:'snapshot-scope:old-profile'}),
      (sa:GraphNode:HpoAnnotation {uid:'snapshot-scope:sa',excluded:false}), (ta:GraphNode:HpoAnnotation {uid:'snapshot-scope:ta',excluded:false}),
      (oa:GraphNode:HpoAnnotation {uid:'snapshot-scope:oa',excluded:false}), (term:GraphNode:HpoTerm {uid:'snapshot-scope:term'}),
      (paper:GraphNode:Publication {uid:'snapshot-scope:current-paper',pmid:400001}),
      (oldpaper:GraphNode:Publication {uid:'snapshot-scope:old-paper',pmid:400002}),
      (s)-[:SOURCE]->(source), (s)-[:TARGET]->(target),
      (sp)-[:PROFILE_OF]->(source), (tp)-[:PROFILE_OF]->(target), (op)-[:PROFILE_OF]->(source),
      (sp)-[:IN_SNAPSHOT]->(current), (tp)-[:IN_SNAPSHOT]->(current), (op)-[:IN_SNAPSHOT]->(old),
      (sp)-[:HAS_ANNOTATION]->(sa), (tp)-[:HAS_ANNOTATION]->(ta), (op)-[:HAS_ANNOTATION]->(oa),
      (sa)-[:PHENOTYPE]->(term), (ta)-[:PHENOTYPE]->(term), (oa)-[:PHENOTYPE]->(term),
      (sa)-[:SUPPORTED_BY]->(paper), (oa)-[:SUPPORTED_BY]->(oldpaper)".replace("snapshot-scope", &prefix))).await.unwrap();
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app_with_graph(pool, GraphSettings { uri, ..settings() });
    let token = demo(&app).await;
    let (status, evidence) = json_request(
        &app,
        "GET",
        &format!("/api/graph/evidence?id={prefix}:similarity&kind=similarity"),
        Some(&token),
        json!(null),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(evidence["papers"].as_array().unwrap().len(), 1);
    assert_eq!(
        evidence["papers"][0]["id"],
        format!("{prefix}:current-paper")
    );
}
