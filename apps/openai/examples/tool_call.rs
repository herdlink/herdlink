use std::io::{self, Write};

use futures_util::StreamExt;
use openai::{Client, Event, InputItem, ResponseRequest, Tool};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AddNumbers {
    a: i64,
    b: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let mut request = ResponseRequest::new(
        std::env::var("OPENAI_MODEL")?,
        "Use add_numbers to add 128 and 256, then tell me the result.",
    );
    request.store = Some(false); // Replay history; works with both API keys and auth.json.
    request.tools.push(Tool::function(
        "add_numbers",
        "Add two integers.",
        json!({
            "type": "object",
            "properties": {"a": {"type": "integer"}, "b": {"type": "integer"}},
            "required": ["a", "b"],
            "additionalProperties": false
        }),
    ));

    // Bound the loop: the model can request more tools after receiving results.
    for _ in 0..8 {
        let mut stream = client.stream(&request).await?;
        let mut completed = None;
        while let Some(event) = stream.next().await {
            match event? {
                Event::TextDelta { delta, .. } => {
                    print!("{delta}");
                    io::stdout().flush()?;
                }
                Event::Completed { response } => completed = Some(response),
                _ => {}
            }
        }
        let response = completed.ok_or("missing completed response")?;
        let mut results = Vec::new();
        // Execute each completed call once. Do not also dispatch from delta events.
        for call in response.tool_calls() {
            let result = match call.name.as_str() {
                "add_numbers" => match call.parse_arguments::<AddNumbers>() {
                    Ok(args) => match args.a.checked_add(args.b) {
                        Some(sum) => json!({"sum": sum}),
                        None => json!({"error": "integer overflow"}),
                    },
                    Err(_) => json!({"error": "invalid arguments"}),
                },
                _ => json!({"error": "unknown tool"}),
            };
            eprintln!("Tool {} returned {result}", call.name);
            results.push(InputItem::tool_output(&call.call_id, result.to_string()));
        }
        if results.is_empty() {
            println!();
            return Ok(());
        }
        request = request.continue_from(&response, results);
    }
    Err("tool-call turn limit reached".into())
}
