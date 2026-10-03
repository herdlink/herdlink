use std::io::{self, Write};

use futures_util::StreamExt;
use openai::{Client, Event, ResponseRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let model = std::env::var("OPENAI_MODEL")?;
    let request = ResponseRequest::new(model, "Explain what a community discussion thread is.");
    let mut stream = client.stream(&request).await?;
    while let Some(event) = stream.next().await {
        match event? {
            Event::TextDelta { delta, .. } => {
                print!("{delta}");
                io::stdout().flush()?;
            }
            Event::Completed { response } => eprintln!("\nResponse: {}", response.id),
            _ => {}
        }
    }
    Ok(())
}
