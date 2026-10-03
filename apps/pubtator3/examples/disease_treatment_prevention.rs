use std::num::NonZeroU32;

use pubtator3::{
    AutocompleteRequest, Client, Concept, EntityId, Page, RelationEntityType, RelationFilter,
    RelationType, RelationsRequest, SearchQuery,
};

/// Find treatment/prevention literature for a disease, defaulting to IgA vasculitis.
/// An optional second argument selects a PubTator entity ID when lookup is ambiguous.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let name = args
        .next()
        .unwrap_or_else(|| "Henoch Schonlein purpura".into());
    let selected: Option<EntityId> = args.next().map(|id| id.parse()).transpose()?;
    let client = Client::new()?;
    let candidates = client
        .autocomplete(&AutocompleteRequest::new(&name).concept(Concept::Disease))
        .await?;
    for disease in &candidates {
        println!(
            "Candidate: {} — {} ({:?})",
            disease.id, disease.name, disease.db_id
        );
    }
    let disease = match selected {
        Some(id) => candidates
            .iter()
            .find(|disease| disease.id == id)
            .ok_or("selected entity ID was not returned by autocomplete")?,
        None if candidates.len() == 1 => &candidates[0],
        None if candidates.is_empty() => {
            return Err("no disease match; try another name or spelling".into());
        }
        None => {
            return Err(
                "multiple candidates; pass the intended entity ID as the second argument".into(),
            );
        }
    };
    println!("\nDisease: {} [{}]", disease.name, disease.id);
    println!(
        "These are extracted literature relations and search matches, not clinical recommendations."
    );

    for kind in [RelationType::Treat, RelationType::Prevent] {
        println!("\nExtracted {kind} relations (up to 3):");
        if kind == RelationType::Prevent {
            println!(
                "  PubTator prevent labels can describe disease–variant associations; inspect the cited papers."
            );
        }
        let mut request = RelationsRequest::new(disease.id.clone())
            .relation_type(kind)
            .limit(NonZeroU32::new(3).unwrap());
        if kind == RelationType::Treat {
            request = request.target_type(RelationEntityType::Chemical);
        }
        let relations = client.relations(&request).await?;
        if relations.is_empty() {
            println!("  No extracted relations returned. This does not establish that none exist.");
        }
        for relation in relations {
            println!(
                "  {} --{}--> {} ({} publications)",
                relation.source, relation.relation_type, relation.target, relation.publications
            );
            // Preserve the API's source/target ordering: starting discovery from a
            // disease does not mean it is the subject performing the treatment.
            let query = SearchQuery::relation(
                RelationFilter::Type(relation.relation_type),
                &relation.source,
                &relation.target,
            );
            print_papers(&client, &query, 2).await?;
        }
    }

    // Retrieval by keywords complements extraction: relevant papers might not carry
    // a treat/prevent annotation, and a paper can discuss prevention of complications.
    for (label, text) in [
        ("Treatment", "treatment OR therapy"),
        ("Prevention", "prevention OR prophylaxis OR recurrence"),
    ] {
        println!("\n{label} keyword search (up to 5 papers):");
        let query = SearchQuery::entity(&disease.id).and(SearchQuery::text(text)?);
        println!("  Query: {query}");
        print_papers(&client, &query, 5).await?;
    }
    Ok(())
}

async fn print_papers(client: &Client, query: &SearchQuery, limit: usize) -> pubtator3::Result<()> {
    let response = client.search(query, Page::FIRST).await?;
    println!("    {} matching publications", response.count);
    for article in response.results.into_iter().take(limit) {
        println!("    PMID {}: {}", article.pmid, article.title);
        println!("      https://pubmed.ncbi.nlm.nih.gov/{}/", article.pmid);
    }
    Ok(())
}
