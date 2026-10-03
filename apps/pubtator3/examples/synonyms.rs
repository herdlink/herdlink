use pubtator3::{AutocompleteRequest, Client, Concept};

/// Resolve a disease name/synonym, then list the selected descriptor's MeSH terms.
/// Pass an optional PubTator entity ID to disambiguate multiple candidates.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| "Lou Gehrig disease".into());
    let selected = args.next();
    let client = Client::new()?;
    let candidates = client
        .autocomplete(&AutocompleteRequest::new(&name).concept(Concept::Disease))
        .await?;
    for entity in &candidates {
        println!(
            "Candidate: {} — {} ({:?})",
            entity.id, entity.name, entity.db_id
        );
    }
    let entity = match selected {
        Some(id) => candidates
            .iter()
            .find(|entity| entity.id.as_str() == id)
            .ok_or("selected entity ID is not among the candidates")?,
        None if candidates.len() == 1 => &candidates[0],
        None if candidates.is_empty() => return Err("no disease match".into()),
        None => {
            return Err(
                "multiple candidates; pass the intended entity ID as the second argument".into(),
            );
        }
    };
    let result = client.synonyms(entity).await?;
    if let Some(term) = result.preferred_term() {
        println!("\nPreferred: {} ({})", term.label, result.descriptor);
    }
    println!("MeSH entry terms (may include narrower concepts):");
    for synonym in result.synonyms() {
        println!("  {} ({})", synonym.label, synonym.id);
    }
    Ok(())
}
