use std::num::NonZeroU32;

use pubtator3::{AutocompleteRequest, Client, Concept, Page, SearchQuery};

/// Normalize a gene name, then paginate publications mentioning it.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    let name = std::env::args().nth(1).unwrap_or_else(|| "BRAF".into());
    let entities = client
        .autocomplete(
            &AutocompleteRequest::new(&name)
                .concept(Concept::Gene)
                .limit(NonZeroU32::new(5).unwrap()),
        )
        .await?;
    for entity in &entities {
        println!("{}: {} ({:?})", entity.id, entity.name, entity.description);
    }
    // Prefer an exact name match: autocomplete suggestions can include other genes.
    let entity = entities
        .iter()
        .find(|entity| entity.name.eq_ignore_ascii_case(&name))
        .ok_or("no exact gene match; inspect suggestions above")?;
    let query = SearchQuery::entity(&entity.id);
    let mut page = Page::FIRST;
    for _ in 0..2 {
        let response = client.search(&query, page).await?;
        println!(
            "Page {} of {}, {} total hits",
            response.current.get(),
            response.total_pages,
            response.count
        );
        for article in response.results.iter() {
            println!("{}  {}", article.pmid, article.title);
        }
        match response.next_page() {
            Some(next) => page = next,
            None => break,
        }
    }
    Ok(())
}
