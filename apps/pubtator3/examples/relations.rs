use std::num::NonZeroU32;

use pubtator3::{
    Client, EntityId, Page, RelationEntityType, RelationFilter, RelationType, RelationsRequest,
    SearchQuery,
};

/// Discover diseases linked to doxorubicin treatment and retrieve supporting papers.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    let drug: EntityId = "@CHEMICAL_Doxorubicin".parse()?;
    let request = RelationsRequest::new(drug.clone())
        .target_type(RelationEntityType::Disease)
        .relation_type(RelationType::Treat)
        .limit(NonZeroU32::new(3).unwrap());
    for relation in client.relations(&request).await? {
        println!(
            "{} -> {}: {} publications",
            relation.source, relation.target, relation.publications
        );
        let query = SearchQuery::relation(
            RelationFilter::Type(relation.relation_type),
            &drug,
            &relation.target,
        );
        for article in client
            .search(&query, Page::FIRST)
            .await?
            .results
            .iter()
            .take(3)
        {
            println!(
                "  https://pubmed.ncbi.nlm.nih.gov/{}/  {}",
                article.pmid, article.title
            );
        }
    }
    Ok(())
}
