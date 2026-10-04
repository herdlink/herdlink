//! Small, repeatable real-data seed. Cached clients persist responses and provenance.
use biomedical_graph::{CachePolicy, CachedHpo, CachedPubTator, Store, tools::GraphTools};
use serde_json::json;
use std::{collections::HashSet, env, fs, io, path::PathBuf};

const DEFAULT_DISEASES: &str =
    "Huntington disease\nParkinson disease\nAmyotrophic lateral sclerosis";
const USAGE: &str = "Usage: seed_use_cases [--diseases-file PATH] [--list]\n\
    Files contain one exact PubTator disease name per line; blank lines and # comments are ignored.\n\
    --list prints the selected names without connecting to Neo4j or fetching data.";

fn parse_diseases(input: &str) -> Result<Vec<String>, io::Error> {
    let mut seen = HashSet::new();
    let mut names = Vec::new();
    for line in input.lines() {
        let name = line.trim();
        if name.is_empty() || name.starts_with('#') {
            continue;
        }
        if name.len() > 200 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Disease names must be at most 200 bytes",
            ));
        }
        if seen.insert(name.to_ascii_lowercase()) {
            names.push(name.to_owned());
        }
    }
    if names.is_empty() || names.len() > 200 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Select between 1 and 200 distinct disease names",
        ));
    }
    Ok(names)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut file = None;
    let mut list = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--diseases-file" => {
                file = Some(args.next().ok_or("--diseases-file requires a path")?);
            }
            "--list" => list = true,
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {arg}\n{USAGE}").into()),
        }
    }
    let input = match file {
        Some(path) => fs::read_to_string(path)?,
        None => DEFAULT_DISEASES.to_owned(),
    };
    let names = parse_diseases(&input)?;
    if list {
        for name in names {
            println!("{name}");
        }
        return Ok(());
    }
    println!("Seeding {} disease use cases", names.len());
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
    if !hpo {
        println!(
            "No HPO dataset configured: seeding PubTator data only. Set HPO_DATA_DIR or download phenotype-data with just graph-data for phenotype comparisons."
        );
    }
    let mut seeded = 0;
    let mut skipped = 0;
    let mut failed = 0;
    for (index, name) in names.iter().enumerate() {
        println!("[{}/{}] {name}", index + 1, names.len());
        match seed_disease(&tools, name, hpo).await {
            Ok(true) => seeded += 1,
            Ok(false) => skipped += 1,
            Err(error) => {
                eprintln!("Failed {name}: {error}");
                failed += 1;
            }
        }
    }
    println!("Finished: {seeded} seeded, {skipped} skipped, {failed} failed");
    if skipped > 0 || failed > 0 {
        return Err("Some diseases were not fully seeded; check the messages above. Rerunning reuses existing nodes and cached results.".into());
    }
    Ok(())
}

async fn seed_disease(
    tools: &GraphTools,
    name: &str,
    hpo: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
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
        return Ok(false);
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
        match tools
            .execute(
                "hpo_similar_diseases",
                &json!({"query":name,"selected":id,"limit":2,"min_score":0.1,"min_phenotypes":3})
                    .to_string(),
            )
            .await
        {
            Ok(report) => println!(
                "{name}: {} phenotype matches persisted",
                report["matches"].as_array().map_or(0, Vec::len)
            ),
            Err(error) => eprintln!("{name}: phenotype data unavailable: {error}"),
        }
    }
    println!(
        "Seeded {name}: entity, up to two gene associations, up to one page of relation papers and one annotated abstract when available"
    );
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_ignore_comments_and_deduplicate_without_guessing() {
        assert_eq!(
            parse_diseases(
                "# A list\n\n Alzheimer Disease \nAlzheimer disease\nCystic Fibrosis\r\n"
            )
            .unwrap(),
            ["Alzheimer Disease", "Cystic Fibrosis"]
        );
        assert!(parse_diseases("# Nothing selected\n").is_err());
        assert!(parse_diseases(&"x".repeat(201)).is_err());
        let too_many = (0..201)
            .map(|i| format!("Disease {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(parse_diseases(&too_many).is_err());
    }

    #[test]
    fn expanded_list_adds_fifty_distinct_diseases_to_the_default_three() {
        let names = parse_diseases(include_str!("more-diseases.txt")).unwrap();
        let defaults = parse_diseases(DEFAULT_DISEASES).unwrap();
        assert_eq!(names.len(), 50);
        assert!(names.iter().all(|name| !defaults.contains(name)));
    }
}
