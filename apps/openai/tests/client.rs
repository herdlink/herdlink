use std::{
    collections::VecDeque,
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Response as HttpResponse},
    routing::post,
};
use futures_util::{StreamExt, stream};
use openai::{Client, Error, Event, InputItem, ResponseRequest, Tool, ToolChoice};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::mpsc;

type CapturedRequest = (HeaderMap, Value);

struct Server {
    client: Client,
    base_url: String,
    requests: mpsc::UnboundedReceiver<CapturedRequest>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Clone)]
struct ServerState {
    replies: Arc<Mutex<VecDeque<HttpResponse<Body>>>>,
    requests: mpsc::UnboundedSender<CapturedRequest>,
}

async fn handler(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> HttpResponse<Body> {
    state.requests.send((headers, body)).unwrap();
    state
        .replies
        .lock()
        .unwrap()
        .pop_front()
        .expect("unexpected request")
}

async fn server(replies: Vec<HttpResponse<Body>>) -> Server {
    let (tx, rx) = mpsc::unbounded_channel();
    let app = Router::new()
        .route("/v1/responses", post(handler))
        .with_state(ServerState {
            replies: Arc::new(Mutex::new(replies.into())),
            requests: tx,
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}/v1/", listener.local_addr().unwrap());
    let client = Client::builder("test-key")
        .base_url(&base_url)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Server {
        client,
        base_url,
        requests: rx,
        task,
    }
}

fn response(output: Value) -> Value {
    json!({"id": "resp_test", "status": "completed", "output": output})
}

fn message(text: &str) -> Value {
    json!({"type": "message", "id": "msg_1", "role": "assistant", "content": [{"type": "output_text", "text": text}]})
}

fn call(id: &str, args: &str) -> Value {
    json!({"type": "function_call", "id": format!("fc_{id}"), "call_id": id, "name": "add", "arguments": args})
}

fn sse(event: Value) -> String {
    format!(
        "event: {}\r\ndata: {event}\r\n\r\n",
        event["type"].as_str().unwrap()
    )
}

fn streamed(text: String) -> HttpResponse<Body> {
    // Split every byte, including inside UTF-8 characters and CRLF delimiters.
    let chunks: Vec<Result<Bytes, Infallible>> = text
        .bytes()
        .map(|byte| Ok(Bytes::from(vec![byte])))
        .collect();
    HttpResponse::builder()
        .header("content-type", "text/event-stream; charset=utf-8")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap()
}

fn json_reply(value: Value) -> HttpResponse<Body> {
    HttpResponse::builder()
        .header("content-type", "application/json")
        .body(Body::from(value.to_string()))
        .unwrap()
}

#[tokio::test]
async fn normal_response_and_request_serialization() {
    let mut server = server(vec![json_reply(response(json!([message("Hello")])))]).await;
    let mut request = ResponseRequest::new("test-model", "Hello");
    request.tools.push(Tool::function(
        "add",
        "Add numbers",
        json!({"type": "object", "properties": {}, "required": [], "additionalProperties": false}),
    ));
    request.tool_choice = Some(ToolChoice::Function("add".into()));
    let result = server.client.create(&request).await.unwrap();
    assert_eq!(result.output_text(), "Hello");
    let (headers, body) = server.requests.recv().await.unwrap();
    assert_eq!(headers["authorization"], "Bearer test-key");
    assert_eq!(body["stream"], false);
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["input"], "Hello");
    assert_eq!(body["tools"][0]["name"], "add");
    assert_eq!(body["tools"][0]["strict"], true);
    assert!(body["tools"][0].get("function").is_none());
    assert_eq!(
        body["tool_choice"],
        json!({"type": "function", "name": "add"})
    );
    assert!(body.get("previous_response_id").is_none());
}

#[tokio::test]
async fn fragmented_stream_with_interleaved_calls_and_continuation() {
    let calls = [call("a", r#"{"a":1,"b":2}"#), call("b", r#"{"a":3,"b":4}"#)];
    let mut wire = ": keepalive\r\n\r\n".to_owned();
    wire += &sse(json!({"type": "response.future_event", "payload": 123}));
    wire += &sse(
        json!({"type": "response.output_text.delta", "item_id": "msg_1", "output_index": 0, "content_index": 0, "delta": "Grüße 🐑"}),
    );
    for (index, item) in calls.iter().enumerate() {
        let mut item = item.clone();
        item["arguments"] = "".into();
        wire += &sse(
            json!({"type": "response.output_item.added", "output_index": index + 1, "item": item}),
        );
    }
    for delta in ["{\"a\":", "1,\"b\":2}"] {
        for (index, item) in calls.iter().enumerate() {
            let delta = if index == 1 {
                delta.replace('1', "3").replace('2', "4")
            } else {
                delta.to_owned()
            };
            wire += &sse(
                json!({"type": "response.function_call_arguments.delta", "item_id": item["id"], "output_index": index + 1, "delta": delta}),
            );
        }
    }
    for (index, item) in calls.iter().enumerate() {
        wire += &sse(
            json!({"type": "response.function_call_arguments.done", "item_id": item["id"], "output_index": index + 1, "arguments": item["arguments"]}),
        );
        wire += &sse(
            json!({"type": "response.output_item.done", "output_index": index + 1, "item": item}),
        );
    }
    wire += &sse(json!({"type": "response.completed", "response": response(json!(calls))}));
    let mut server = server(vec![
        streamed(wire),
        json_reply(response(json!([message("3 and 7")]))),
    ])
    .await;
    let mut request = ResponseRequest::new("test-model", "Add two pairs");
    request.instructions = Some("Use tools".into());
    request.tools.push(Tool::function("add", "Add", json!({})));
    let mut stream = server.client.stream(&request).await.unwrap();
    let mut text = String::new();
    let mut arguments = [String::new(), String::new()];
    let mut completed = None;
    let mut finished_calls = 0;
    while let Some(event) = stream.next().await {
        match event.unwrap() {
            Event::TextDelta { delta, .. } => text += &delta,
            Event::FunctionArgumentsDelta {
                output_index,
                item_id,
                delta,
            } => {
                assert_eq!(item_id, if output_index == 1 { "fc_a" } else { "fc_b" });
                arguments[output_index - 1] += &delta;
            }
            Event::OutputItemDone {
                item: openai::OutputItem::FunctionCall(_),
                ..
            } => finished_calls += 1,
            Event::Completed { response } => completed = Some(response),
            _ => {}
        }
    }
    assert_eq!(text, "Grüße 🐑");
    assert_eq!(finished_calls, 2);
    let response = completed.unwrap();
    #[derive(Deserialize)]
    struct Args {
        a: i64,
        b: i64,
    }
    let results: Vec<_> = response
        .tool_calls()
        .enumerate()
        .map(|(index, call)| {
            assert_eq!(call.arguments, arguments[index]);
            let args: Args = call.parse_arguments().unwrap();
            InputItem::tool_output(&call.call_id, (args.a + args.b).to_string())
        })
        .collect();
    request = request.continue_from(&response, results);
    assert_eq!(
        server.client.create(&request).await.unwrap().output_text(),
        "3 and 7"
    );
    let (_, first) = server.requests.recv().await.unwrap();
    let (_, next) = server.requests.recv().await.unwrap();
    assert_eq!(first["stream"], true);
    assert_eq!(next["previous_response_id"], "resp_test");
    assert_eq!(
        next["input"],
        json!([
            {"type": "function_call_output", "call_id": "a", "output": "3"},
            {"type": "function_call_output", "call_id": "b", "output": "7"}
        ])
    );
    assert_eq!(next["instructions"], first["instructions"]);
    assert_eq!(next["tools"], first["tools"]);
}

#[tokio::test]
async fn yields_text_before_server_finishes() {
    for content_type in [Some("text/event-stream"), None] {
        let (tx, rx) = mpsc::unbounded_channel::<Result<Bytes, Infallible>>();
        let body = Body::from_stream(stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|item| (item, rx))
        }));
        let mut reply = HttpResponse::builder();
        if let Some(content_type) = content_type {
            reply = reply.header("content-type", content_type);
        }
        let reply = reply.body(body).unwrap();
        let server = server(vec![reply]).await;
        tx.send(Ok(Bytes::from(sse(json!({"type": "response.output_text.delta", "item_id": "msg_1", "output_index": 0, "content_index": 0, "delta": "first"}))))).unwrap();
        let mut stream = server
            .client
            .stream(&ResponseRequest::new("test", "hello"))
            .await
            .unwrap();
        let first = tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(first, Event::TextDelta { delta, .. } if delta == "first"));
        // Completion has not been sent yet: buffering the body would deadlock above.
        tx.send(Ok(Bytes::from(sse(
            json!({"type": "response.completed", "response": response(json!([]))}),
        ))))
        .unwrap();
        assert!(matches!(
            stream.next().await.unwrap().unwrap(),
            Event::Completed { .. }
        ));
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
async fn missing_content_type_still_requires_valid_completed_events() {
    for wire in [
        String::new(),
        "data: [DONE]\n\n".into(),
        r#"{"error":{"message":"not an event stream"}}"#.into(),
        "<html>Not an event stream</html>".into(),
    ] {
        let mut reply = streamed(wire);
        reply.headers_mut().remove("content-type");
        let server = server(vec![reply]).await;
        let mut events = server
            .client
            .stream(&ResponseRequest::new("test", "hi"))
            .await
            .unwrap();
        assert!(matches!(
            events.next().await.unwrap(),
            Err(Error::UnexpectedEof)
        ));
        assert!(events.next().await.is_none());
    }
    let mut reply = streamed(sse(
        json!({"type": "error", "code": "server_error", "message": "try later"}),
    ));
    reply.headers_mut().remove("content-type");
    let server = server(vec![reply]).await;
    let mut events = server
        .client
        .stream(&ResponseRequest::new("test", "hi"))
        .await
        .unwrap();
    assert!(matches!(
        events.next().await.unwrap(),
        Err(Error::Api { .. })
    ));
}

#[tokio::test]
async fn http_errors_preserve_status_request_id_and_body() {
    let reply = HttpResponse::builder()
        .status(429)
        .header("x-request-id", "req_123")
        .body(Body::from(r#"{"error":{"message":"rate limited"}}"#))
        .unwrap();
    let server = server(vec![reply]).await;
    let error = server
        .client
        .create(&ResponseRequest::new("test", "hi"))
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::Http { status: 429, request_id: Some(id), body } if id == "req_123" && body.contains("rate limited"))
    );
}

#[tokio::test]
async fn truncated_and_malformed_streams_are_errors() {
    for (wire, malformed) in [
        (String::new(), false),
        ("data: [DONE]\n\n".into(), false),
        ("data: not-json\n\n".into(), true),
    ] {
        let server = server(vec![streamed(wire)]).await;
        let mut stream = server
            .client
            .stream(&ResponseRequest::new("test", "hi"))
            .await
            .unwrap();
        let error = stream.next().await.unwrap().unwrap_err();
        if malformed {
            assert!(matches!(error, Error::Json(_)));
        } else {
            assert!(matches!(error, Error::UnexpectedEof));
        }
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
async fn failed_and_incomplete_responses_retain_details() {
    for status in ["failed", "incomplete"] {
        let mut value = response(json!([]));
        value["status"] = status.into();
        value["error"] = json!({"code": "server_error", "message": "failure"});
        value["incomplete_details"] = json!({"reason": "max_output_tokens"});
        let server = server(vec![
            streamed(sse(
                json!({"type": format!("response.{status}"), "response": value}),
            )),
            json_reply(value),
        ])
        .await;
        let request = ResponseRequest::new("test", "hi");
        let mut stream = server.client.stream(&request).await.unwrap();
        for error in [
            stream.next().await.unwrap().unwrap_err(),
            server.client.create(&request).await.unwrap_err(),
        ] {
            let Error::UnsuccessfulResponse(response) = error else {
                panic!("wrong error")
            };
            assert_eq!(response.status, status);
            assert_eq!(
                response.incomplete_details.unwrap()["reason"],
                "max_output_tokens"
            );
            assert_eq!(response.error.unwrap()["code"], "server_error");
        }
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
async fn api_errors_and_wrong_content_type_are_reported() {
    let server = server(vec![
        streamed(sse(
            json!({"type": "error", "code": "server_error", "message": "try later"}),
        )),
        json_reply(response(json!([]))),
    ])
    .await;
    let request = ResponseRequest::new("test", "hi");
    let mut stream = server.client.stream(&request).await.unwrap();
    assert!(
        matches!(stream.next().await.unwrap().unwrap_err(), Error::Api { code: Some(code), message } if code == "server_error" && message == "try later")
    );
    assert!(stream.next().await.is_none());
    assert!(matches!(
        server.client.stream(&request).await,
        Err(Error::Stream(_))
    ));
}

#[test]
fn rejects_invalid_configuration_and_tool_arguments() {
    assert!(Client::new("").is_err());
    assert!(Client::new("key\ninvalid").is_err());
    for url in [
        "not a url",
        "ftp://localhost",
        "http://localhost?query=1",
        "http://user:pass@localhost",
    ] {
        assert!(Client::builder("key").base_url(url).build().is_err());
    }
    let call: openai::FunctionCall = serde_json::from_value(call("a", "{")).unwrap();
    assert!(call.parse_arguments::<Value>().is_err());
}

fn auth_file(value: Value) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), value.to_string()).unwrap();
    file
}

fn codex_auth() -> Value {
    json!({
        "auth_mode": "chatgpt",
        "OPENAI_API_KEY": null,
        "tokens": {
            "access_token": "test-access-token",
            "account_id": "test-account-id",
            "id_token": "unused-id-token",
            "refresh_token": "unused-refresh-token"
        },
        "last_refresh": "2026-10-04T00:00:00Z"
    })
}

#[tokio::test]
async fn codex_auth_streams_and_replays_tools_with_encrypted_reasoning() {
    let reasoning = json!({"type": "reasoning", "id": "rs_1", "summary": [], "encrypted_content": "opaque-reasoning"});
    let tool = call("call_1", r#"{"a":1,"b":2}"#);
    let output = json!([reasoning, tool]);
    let mut replies = vec![
        // The live Codex endpoint can omit both the Content-Type header and
        // the terminal output list. Preserve items, including unknown reasoning
        // fields, and order them by output_index rather than arrival order.
        streamed(
            sse(json!({"type": "response.output_item.done", "output_index": 1, "item": output[1]}))
                + &sse(
                    json!({"type": "response.output_item.done", "output_index": 0, "item": output[0]}),
                )
                + &sse(json!({"type": "response.completed", "response": response(json!([]))})),
        ),
        streamed(
            sse(
                json!({"type": "response.output_item.done", "output_index": 0, "item": message("3")}),
            ) + &sse(json!({"type": "response.completed", "response": response(json!([]))})),
        ),
    ];
    for reply in &mut replies {
        reply.headers_mut().remove("content-type");
    }
    let mut server = server(replies).await;
    let file = auth_file(codex_auth());
    let client = Client::builder_from_auth_file(file.path())
        .unwrap()
        .base_url(&server.base_url)
        .build()
        .unwrap();
    let mut request = ResponseRequest::new("test-model", "Add 1 and 2");
    request
        .tools
        .push(Tool::function("add", "Add numbers", json!({})));
    let mut stream = client.stream(&request).await.unwrap();
    for _ in 0..2 {
        assert!(matches!(
            stream.next().await.unwrap().unwrap(),
            Event::OutputItemDone { .. }
        ));
    }
    let Event::Completed { response: first } = stream.next().await.unwrap().unwrap() else {
        panic!("expected response")
    };
    assert_eq!(first.store, Some(false));
    assert_eq!(first.tool_calls().next().unwrap().call_id, "call_1");
    assert!(stream.next().await.is_none());
    request = request.continue_from(&first, vec![InputItem::tool_output("call_1", "3")]);
    // create() must also use SSE on the wire for Codex credentials.
    let second = client.create(&request).await.unwrap();
    assert_eq!(second.output_text(), "3");
    let (headers, first_body) = server.requests.recv().await.unwrap();
    assert_eq!(headers["authorization"], "Bearer test-access-token");
    assert_eq!(headers["chatgpt-account-id"], "test-account-id");
    assert!(!first_body.to_string().contains("unused-refresh-token"));
    assert_eq!(first_body["stream"], true);
    assert_eq!(first_body["store"], false);
    assert_eq!(first_body["instructions"], "");
    assert_eq!(
        first_body["include"],
        json!(["reasoning.encrypted_content"])
    );
    assert_eq!(
        first_body["input"],
        json!([{"type": "message", "role": "user", "content": "Add 1 and 2"}])
    );
    let (_, second_body) = server.requests.recv().await.unwrap();
    assert_eq!(second_body["stream"], true);
    assert_eq!(second_body["store"], false);
    assert!(second_body.get("previous_response_id").is_none());
    assert_eq!(second_body["input"][0], first_body["input"][0]);
    assert_eq!(second_body["input"][1], output[0]);
    assert_eq!(second_body["input"][2], output[1]);
    assert_eq!(
        second_body["input"][3],
        json!({"type": "function_call_output", "call_id": "call_1", "output": "3"})
    );
    assert_eq!(second_body["tools"], first_body["tools"]);
    let third = request.continue_from(
        &second,
        vec![InputItem::message(openai::Role::User, "Thanks")],
    );
    let third = serde_json::to_value(third).unwrap();
    assert_eq!(third["input"].as_array().unwrap().len(), 6);
    assert_eq!(third["input"][4], message("3"));
}

#[tokio::test]
async fn api_key_auth_file_preserves_normal_api_behavior() {
    for value in [
        json!({"OPENAI_API_KEY": "file-key"}),
        json!({"api_key": "file-key"}),
        json!({"auth_mode": "apikey", "OPENAI_API_KEY": "file-key"}),
    ] {
        let file = auth_file(value);
        let mut server = server(vec![json_reply(response(json!([])))]).await;
        let client = Client::builder_from_auth_file(file.path())
            .unwrap()
            .base_url(&server.base_url)
            .build()
            .unwrap();
        client
            .create(&ResponseRequest::new("test", "hi"))
            .await
            .unwrap();
        let (headers, body) = server.requests.recv().await.unwrap();
        assert_eq!(headers["authorization"], "Bearer file-key");
        assert!(!headers.contains_key("chatgpt-account-id"));
        assert_eq!(body["stream"], false);
    }
}

#[test]
fn invalid_auth_files_fail_without_leaking_secrets() {
    for value in [
        json!({}),
        json!({"auth_mode": "secret-unknown-mode"}),
        json!({"OPENAI_API_KEY": ""}),
        json!({"auth_mode": "apikey", "tokens": {"access_token": "secret-token", "account_id": "account"}}),
        json!({"auth_mode": "chatgpt", "OPENAI_API_KEY": "secret-key"}),
        json!({"tokens": {"access_token": "secret-token"}}),
        json!({"tokens": {"access_token": "secret-token", "account_id": ""}}),
        json!({"tokens": {"access_token": ["secret-token"], "account_id": "account"}}),
        json!({"OPENAI_API_KEY": "secret-key\ninjected"}),
        json!({"tokens": {"access_token": "secret-token", "account_id": "secret-account\ninjected"}}),
    ] {
        let file = auth_file(value);
        let Err(error) = Client::from_auth_file(file.path()) else {
            panic!("expected an error")
        };
        assert!(matches!(error, Error::Configuration(_)));
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), br#"{"OPENAI_API_KEY": "secret" BROKEN}"#).unwrap();
    assert!(matches!(
        Client::from_auth_file(file.path()),
        Err(Error::Configuration(_))
    ));
    let missing = file.path().to_owned();
    drop(file);
    assert!(matches!(
        Client::from_auth_file(missing),
        Err(Error::AuthFileIo(_))
    ));
}

#[tokio::test]
async fn codex_rejects_incompatible_request_options_before_sending() {
    let server = server(vec![]).await;
    let file = auth_file(codex_auth());
    let client = Client::builder_from_auth_file(file.path())
        .unwrap()
        .base_url(&server.base_url)
        .build()
        .unwrap();
    for option in 0..3 {
        let mut request = ResponseRequest::new("test", "hi");
        match option {
            0 => request.store = Some(true),
            1 => request.previous_response_id = Some("resp_previous".into()),
            _ => request.max_output_tokens = Some(10),
        }
        assert!(matches!(
            client.stream(&request).await,
            Err(Error::Configuration(_))
        ));
    }
}

// Exercise environment selection in a subprocess so parallel tests never mutate
// the parent process's environment or accidentally load real credentials.
#[tokio::test]
async fn auth_file_environment_takes_precedence_over_api_key() {
    let file = auth_file(codex_auth());
    let mut server = server(vec![streamed(sse(
        json!({"type": "response.completed", "response": response(json!([]))}),
    ))])
    .await;
    let mut child = std::process::Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", "auth_environment_child"])
        .env("HERDLINK_AUTH_TEST_CHILD", "1")
        .env("OPENAI_AUTH_FILE", file.path())
        .env("OPENAI_API_KEY", "should-not-be-used")
        .env("OPENAI_BASE_URL", &server.base_url);
    let output = tokio::task::spawn_blocking(move || child.output().unwrap())
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let (headers, _) = server.requests.recv().await.unwrap();
    assert_eq!(headers["authorization"], "Bearer test-access-token");
}

#[tokio::test]
async fn auth_environment_child() {
    if std::env::var_os("HERDLINK_AUTH_TEST_CHILD").is_none() {
        return;
    }
    Client::from_env()
        .unwrap()
        .create(&ResponseRequest::new("test", "hi"))
        .await
        .unwrap();
}
