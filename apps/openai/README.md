# openai

Local async client for the OpenAI **Responses API**. Supports normal responses,
SSE text streaming, function tools, streamed function arguments, and tool-result
continuation. The client reuses an HTTP connection pool and uses Rustls for TLS.

Add it to another workspace crate:

```toml
[dependencies]
openai = { workspace = true }
```

```rust,no_run
use openai::{Client, ResponseRequest};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::from_env()?;
let request = ResponseRequest::new(std::env::var("OPENAI_MODEL")?, "Hello!");
let response = client.create(&request).await?;
println!("{}", response.output_text());
# Ok(())
# }
```

Set `OPENAI_MODEL` and either `OPENAI_API_KEY` or `OPENAI_AUTH_FILE` to run either example:

```sh
cargo run -p openai --example stream
cargo run -p openai --example tool_call
```

`OPENAI_BASE_URL` optionally overrides the API root. With an API key it defaults
to `https://api.openai.com/v1`; with Codex ChatGPT credentials it defaults to
`https://chatgpt.com/backend-api/codex`. These examples make real requests and
consume the selected account's usage allowance. `.env` is not loaded
automatically. `Client::builder(key).base_url(url).timeout(duration).build()`
configures the client directly. The default total request timeout is five minutes,
including streaming, with a 30-second connection timeout.

## Codex auth.json

Pass the path to a Codex login file instead of passing an API key:

```rust,no_run
# fn example() -> openai::Result<()> {
let client = openai::Client::from_auth_file("/path/to/auth.json")?;
# Ok(())
# }
```

Or use the existing examples with your Codex login:

```sh
OPENAI_AUTH_FILE="$HOME/.codex/auth.json" cargo run -p openai --example tool_call
```

Keep `OPENAI_MODEL` set to a model available to your account. `OPENAI_AUTH_FILE`
takes precedence over `OPENAI_API_KEY`; an invalid file is an error and never
silently falls back to another credential. Paths passed in Rust or environment
variables are literal; expand `~` in your shell or supply an absolute path.
`Client::builder_from_auth_file(path)?.base_url(url).build()?` allows configuration.

The loader accepts Codex's `tokens.access_token` and `tokens.account_id` for
ChatGPT login, or `OPENAI_API_KEY` (`api_key` also accepted) for API-key files.
An explicit `auth_mode` of `chatgpt` or `apikey` selects that credential type.
Refresh tokens and ID tokens are not sent with inference requests. Credentials
are read once when constructing the client; this crate does not refresh tokens
or modify the file. After Codex renews your login, construct a new client. Login
files stored only in an OS keychain are not supported by this file loader.

With ChatGPT credentials, the client sends the account header and uses streaming
with `store: false`. `create()` collects that stream into a completed response.
Tool continuation automatically replays the input and complete output items,
including encrypted reasoning, before appending tool results. Explicit
`store: true`, `previous_response_id`, and `max_output_tokens` are rejected for
this backend. API-key behavior remains available as before.

This is compatibility with Codex's login-file format and backend, not the separate
Sign in with ChatGPT flow for registering a third-party OAuth app. Backend
compatibility is covered by local fixtures and has also been checked with the
streaming and tool-call examples against a live ChatGPT login using `gpt-6-luna`.
Official OpenAI documentation describes [Codex login-file storage](https://learn.chatgpt.com/docs/auth#login-caching).

## Streaming and tools

- `client.stream(&request).await?` returns a stream of `Result<Event>`; consume it
  using `futures_util::StreamExt::next`.
- `Event::TextDelta` supplies text immediately. Function-argument deltas include
  an item ID and output index so callers can track interleaved calls.
- `Event::OutputItemDone` contains completed items. `Event::Completed` contains
  the complete response, including all function calls. Use one of these to
  dispatch tools; dispatching from both would execute a call twice.
- `response.tool_calls()` supplies names, call IDs, and JSON argument strings.
  `call.parse_arguments::<YourType>()` deserializes a completed call.
- Declare functions with `Tool::function(name, description, json_schema)`.
  This defaults to strict mode; schemas must require all properties and set
  `additionalProperties: false` on objects. Nullable properties can represent
  optional values. `ToolChoice` supports auto, none, required, or a named function.
- Return each result with `InputItem::tool_output(call.call_id, result_string)`.
  `request.continue_from(&response, results)` retains tools and instructions.
  Stored responses use `previous_response_id`; unstored responses replay the
  conversation. The tool example uses `store: false` for both authentication modes.
- The caller validates arguments, authorizes and executes tools, and bounds the
  loop. No functions execute automatically. See `examples/tool_call.rs` for a
  complete streamed conversation with multiple tool results per turn.

HTTP errors retain status, request ID, and response body. API stream errors,
failed/incomplete responses, malformed events, and premature EOF are errors.
Some Codex responses omit `Content-Type`; the client accepts those and validates
the streamed events. Explicitly incompatible content types are rejected. If a
terminal response omits its output items, the client reconstructs them from
`response.output_item.done` events in output-index order.
An unsuccessful response is retained in `Error::UnsuccessfulResponse` so callers
can inspect its error, incomplete details, and partial output. New unrecognized
event types are skipped; unrecognized output types become `Other`. Refusals are
available in `OutputContent::Refusal` and are not included in `output_text()`.
Dropping a stream closes the local connection. There are no automatic retries.

This crate currently covers text and custom function tools on `/v1/responses`.
Chat Completions, image/audio input, built-in tools, and automatic Rust-function
registration are outside its current scope. It is available as a workspace
dependency; no backend route is connected to an AI model yet.

## Validation

```sh
cargo test -p openai
```

Tests use synthetic credentials and a local HTTP server. They exercise actual
request serialization and SSE parsing, including fragmented UTF-8, interleaved
calls, continuation, auth-file loading, account headers, environment precedence,
and failures.

API references: [streaming Responses](https://developers.openai.com/api/docs/guides/streaming-responses)
and [function calling](https://developers.openai.com/api/docs/guides/function-calling).
