use biomedical_graph::{
    CachePolicy, CachedHpo, CachedPubTator, Store,
    tools::{GraphTools, hpo_tools, pubtator_tools},
};
use openai::{Client, ResponseRequest};
use serde_json::{Value, json};
use std::{env, path::PathBuf, sync::Arc};

fn usage() -> &'static str {
    "Usage: tool_calls [--list | --smoke | --call NAME JSON | --prompt TEXT]\n\
     Default: --smoke (calls all 16 tools and repeats them through new cached clients).\n\
     HPO_DATA_DIR overrides the small bundled HPO fixture dataset.\n\
     --prompt requires OPENAI_MODEL and OPENAI_API_KEY or OPENAI_AUTH_FILE."
}

fn dispatcher(
    store: &Store,
    upstream: &pubtator3::Client,
    dataset: &Arc<pubtator3_hpo::Dataset>,
) -> GraphTools {
    GraphTools::new(CachedPubTator::new(
        upstream.clone(),
        store.clone(),
        CachePolicy::default(),
    ))
    .with_hpo(CachedHpo::new(
        pubtator3_hpo::Client::new(upstream.clone(), dataset.clone()),
        store.clone(),
        CachePolicy::default(),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("--help") => {
            println!("{}", usage());
            return Ok(());
        }
        Some("--list") if arguments.len() == 1 => {
            let mut tools = pubtator_tools();
            tools.extend(hpo_tools());
            println!("{}", serde_json::to_string_pretty(&tools)?);
            return Ok(());
        }
        None => (),
        Some("--smoke") if arguments.len() == 1 => (),
        Some("--call") if arguments.len() == 3 => (),
        Some("--prompt") if arguments.len() == 2 => (),
        _ => return Err(usage().into()),
    }
    let config = neo4rs::ConfigBuilder::default()
        .uri(env::var("NEO4J_URI").unwrap_or_else(|_| "127.0.0.1:7687".into()))
        .user(env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into()))
        .password(env::var("NEO4J_PASSWORD").unwrap_or_default())
        .db(env::var("NEO4J_DATABASE").unwrap_or_else(|_| "neo4j".into()))
        .build()?;
    let store = Store::connect(config).await?;
    let mut builder = pubtator3::Client::builder();
    if let Ok(url) = env::var("PUBTATOR_BASE_URL") {
        builder = builder.base_url(url);
    }
    if let Ok(url) = env::var("MESH_BASE_URL") {
        builder = builder.mesh_base_url(url);
    }
    let upstream = builder.build()?;
    let directory = env::var_os("HPO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")));
    let dataset =
        tokio::task::spawn_blocking(move || pubtator3_hpo::Dataset::from_dir(directory)).await??;
    let dataset = Arc::new(dataset);
    let tools = dispatcher(&store, &upstream, &dataset);
    match arguments.first().map(String::as_str) {
        Some("--call") => {
            let result = tools.execute(&arguments[1], &arguments[2]).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some("--prompt") => {
            let client = Client::from_env()?;
            let mut request = ResponseRequest::new(env::var("OPENAI_MODEL")?, arguments[1].clone());
            request.store = Some(false);
            request.tools = tools.definitions();
            request.instructions = Some("Use the biomedical tools to answer with supporting IDs. Keep searches and exports bounded. The default HPO snapshot is a tiny demonstration subset unless HPO_DATA_DIR is configured; an empty similarity result does not imply no similar diseases exist. Distinguish phenotype similarity, co-mentions and extracted relationships. Do not present publication counts as confidence scores.".into());
            for _ in 0..12 {
                let response = client.create(&request).await?;
                let mut results = Vec::new();
                for call in response.tool_calls() {
                    eprintln!("Calling {}", call.name);
                    results.push(tools.call(call).await);
                }
                if results.is_empty() {
                    println!("{}", response.output_text());
                    return Ok(());
                }
                request = request.continue_from(&response, results);
            }
            return Err("tool-call turn limit reached (12)".into());
        }
        _ => {
            eprintln!("Smoke test: all 16 tools. Cache misses make live PubTator/MeSH requests.");
            eprintln!(
                "HPO corpus: {} diseases (bundled default is incomplete).",
                dataset.disease_count()
            );
            let disease =
                json!({"query":"Huntington disease", "selected":"@DISEASE_Huntington_Disease"});
            let mut calls: Vec<(&str, Value)> = vec![
                (
                    "pubtator_autocomplete",
                    json!({"query":"Huntington disease", "concept":"disease", "limit":2}),
                ),
                (
                    "pubtator_relations",
                    json!({"entity":"@DISEASE_Huntington_Disease", "target_type":"gene", "relation_type":"associate", "limit":2}),
                ),
                (
                    "pubtator_search",
                    json!({"query":"@DISEASE_Huntington_Disease", "page":1}),
                ),
                (
                    "pubtator_annotations",
                    json!({"pmids":["19894120"], "scope":"abstract"}),
                ),
                (
                    "pubtator_annotations_batched",
                    json!({"pmids":["19894120"], "scope":"abstract"}),
                ),
                ("pubtator_pmc_annotations", json!({"pmcids":["PMC6142073"]})),
                ("pubtator_mesh_synonyms", json!({"descriptor":"D006816"})),
                (
                    "pubtator_synonyms",
                    json!({"entity":{"_id":"@DISEASE_Huntington_Disease", "name":"Huntington Disease", "biotype":"disease", "db":"ncbi_mesh", "db_id":"D006816"}}),
                ),
                (
                    "pubtator_export",
                    json!({"pmids":["19894120"], "format":"biocjson", "scope":"abstract"}),
                ),
                (
                    "pubtator_pmc_export",
                    json!({"pmcids":["PMC6142073"], "format":"biocjson"}),
                ),
                ("hpo_resolve_disease", disease.clone()),
                ("hpo_disease_phenotypes", disease),
                ("hpo_profile", json!({"id":"MONDO:0007739"})),
                (
                    "hpo_similar_diseases",
                    json!({"query":"Huntington disease", "selected":"@DISEASE_Huntington_Disease", "limit":2, "min_score":0.0, "min_phenotypes":1}),
                ),
                (
                    "hpo_supporting_papers",
                    json!({"source":"@DISEASE_Huntington_Disease", "target":"@DISEASE_Parkinson_Disease", "page":1}),
                ),
            ];
            let mut outputs = Vec::new();
            for (name, args) in &calls {
                let result = match tools.execute(name, &args.to_string()).await {
                    Ok(result) => result,
                    Err(biomedical_graph::Error::Hpo(pubtator3_hpo::Error::Data(message)))
                        if *name == "hpo_similar_diseases"
                            && message.contains("no informative terms") =>
                    {
                        println!(
                            "SKIP hpo_similar_diseases: this HPO corpus has no informative terms; set HPO_DATA_DIR to a full snapshot."
                        );
                        outputs.push(Value::Null);
                        continue;
                    }
                    Err(error) => return Err(format!("{name}: {error}").into()),
                };
                println!("PASS {name} ({} JSON bytes)", result.to_string().len());
                outputs.push(result);
            }
            // Exercise reverse mapping even with the one-disease fixture corpus,
            // whose similarity report has no other diseases to rank.
            let profile = &outputs[12];
            let reverse = json!({"disease":{
                "disease_id":profile["id"], "name":profile["name"], "mondo":profile["mondo"],
                "annotation_ids":profile["annotation_ids"], "mesh_ids":profile["mesh_ids"],
                "score":1.0, "phenotype_count":profile["phenotypes"].as_array().ok_or("missing phenotypes")?.len(),
                "shared_phenotypes":profile["phenotypes"], "source_only_phenotypes":[], "conflicting_phenotypes":[],
            }});
            let result = tools
                .execute("hpo_pubtator_entities", &reverse.to_string())
                .await?;
            println!(
                "PASS hpo_pubtator_entities ({} JSON bytes)",
                result.to_string().len()
            );
            calls.push(("hpo_pubtator_entities", reverse));
            outputs.push(result);
            let fresh = dispatcher(&store, &upstream, &dataset);
            for ((name, args), expected) in calls.iter().zip(&outputs) {
                if *name == "hpo_similar_diseases" && expected.is_null() {
                    continue;
                }
                let repeated = fresh.execute(name, &args.to_string()).await?;
                if &repeated != expected {
                    return Err(format!("{name}: persisted response differs").into());
                }
            }
            println!(
                "PASS all successful responses round-trip through reconstructed cached clients."
            );
            println!("Cache directory: {}", store.cache_dir().display());
        }
    }
    Ok(())
}
