//! Small, repeatable real-data seed. Cached clients persist responses and provenance.
use biomedical_graph::{CachePolicy, CachedHpo, CachedPubTator, Store, tools::GraphTools};
use serde_json::json;
use std::{env, path::PathBuf};

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
    let mut tools = GraphTools::new(CachedPubTator::new(
        upstream.clone(),
        store.clone(),
        CachePolicy::default(),
    ));
    let directory = env::var_os("HPO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("phenotype-data"));
    if directory.is_dir() {
        let dataset =
            tokio::task::spawn_blocking(move || pubtator3_hpo::Dataset::from_dir(directory))
                .await??;
        println!("HPO corpus: {} disease profiles", dataset.disease_count());
        tools = tools.with_hpo(CachedHpo::new(
            pubtator3_hpo::Client::new(upstream, dataset),
            store,
            CachePolicy::default(),
        ));
    }
    let hpo = tools.definitions().iter().any(
        |tool| matches!(tool,openai::Tool::Function {name,..} if name == "hpo_similar_diseases"),
    );
    for name in [
        "Huntington disease",
        "Parkinson disease",
        "Amyotrophic lateral sclerosis",
    ] {
        let entities = tools
            .execute(
                "pubtator_autocomplete",
                &json!({"query":name,"concept":"disease","limit":3}).to_string(),
            )
            .await?;
        // Resolve the exact normalized name; don't silently use a similarly named disease.
        let entity = entities.as_array().and_then(|items| {
            items.iter().find(|item| {
                item["name"]
                    .as_str()
                    .is_some_and(|value| value.eq_ignore_ascii_case(name))
            })
        });
        let Some(entity) = entity else {
            println!("Skip {name}: no exact PubTator name match");
            continue;
        };
        let id = entity["_id"].as_str().ok_or("missing entity ID")?;
        let relations = tools
            .execute(
                "pubtator_relations",
                &json!({"entity":id,"target_type":"gene","relation_type":"associate","limit":2})
                    .to_string(),
            )
            .await?;
        for relation in relations.as_array().into_iter().flatten().take(1) {
            let papers = tools.execute("pubtator_relation_papers",&json!({"source":relation["source"],"target":relation["target"],"relation_type":relation["type"],"page":1}).to_string()).await?;
            if let Some(pmid) = papers["results"]
                .as_array()
                .and_then(|papers| papers.first())
                .and_then(|paper| paper.get("pmid"))
            {
                tools
                    .execute(
                        "pubtator_annotations",
                        &json!({"pmids":[pmid],"scope":"abstract"}).to_string(),
                    )
                    .await?;
            }
        }
        if hpo {
            match tools.execute("hpo_similar_diseases",&json!({"query":name,"selected":id,"limit":2,"min_score":0.1,"min_phenotypes":3}).to_string()).await {
                Ok(report) => println!("{name}: {} phenotype matches persisted",report["matches"].as_array().map_or(0,Vec::len)),
                Err(error) => eprintln!("{name}: phenotype data unavailable: {error}"),
            }
        }
        println!(
            "Seeded {name}: entity, up to two gene associations, one page of relation papers and one annotated abstract"
        );
    }
    Ok(())
}
