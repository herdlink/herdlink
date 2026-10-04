use biomedical_graph::{CachePolicy, CachedHpo, CachedPubTator, Store};
use pubtator3::{
    AutocompleteRequest, Concept, Pmid, RelationEntityType, RelationType, RelationsRequest,
    TextScope,
};
use std::{env, num::NonZeroU32};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = neo4rs::ConfigBuilder::default()
        .uri(env::var("NEO4J_URI").unwrap_or_else(|_| "127.0.0.1:7687".into()))
        .user(env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into()))
        .password(env::var("NEO4J_PASSWORD").unwrap_or_default())
        .db(env::var("NEO4J_DATABASE").unwrap_or_else(|_| "neo4j".into()))
        .build()?;
    let store = Store::connect(config).await?;
    let upstream = pubtator3::Client::new()?;
    let pubtator = CachedPubTator::new(upstream.clone(), store.clone(), CachePolicy::default());
    let entities = pubtator
        .autocomplete(
            &AutocompleteRequest::new("Huntington disease")
                .concept(Concept::Disease)
                .limit(NonZeroU32::new(2).unwrap()),
        )
        .await?;
    println!("Autocomplete: {} entities", entities.len());
    let id = "@DISEASE_Huntington_Disease".parse()?;
    let relations = pubtator
        .relations(
            &RelationsRequest::new(id)
                .target_type(RelationEntityType::Gene)
                .relation_type(RelationType::Associate)
                .limit(NonZeroU32::new(2).unwrap()),
        )
        .await?;
    for relation in &relations {
        println!(
            "{} -[{}]-> {} ({} publications)",
            relation.source, relation.relation_type, relation.target, relation.publications
        );
    }
    let docs = pubtator
        .annotations(&[Pmid::new(19894120)?], TextScope::Abstract)
        .await?;
    println!(
        "BioC: {} document, {} mentions",
        docs.len(),
        docs.iter().flat_map(|d| d.annotations()).count()
    );
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "apps/biomedical_graph/tests/fixtures".into());
    let dataset =
        tokio::task::spawn_blocking(move || pubtator3_hpo::Dataset::from_dir(path)).await??;
    let hpo = CachedHpo::new(
        pubtator3_hpo::Client::new(upstream, dataset),
        store,
        CachePolicy::default(),
    );
    let mapped = hpo
        .disease_phenotypes(
            "Huntington disease",
            Some(&"@DISEASE_Huntington_Disease".parse()?),
        )
        .await?;
    println!(
        "HPO: {} via {:?}, {} positive features",
        mapped.profile.id,
        mapped.mapping.method,
        mapped.profile.phenotypes.len()
    );
    for term in mapped.profile.phenotypes.iter().take(5) {
        println!("{}: {}", term.id, term.name);
    }
    Ok(())
}
