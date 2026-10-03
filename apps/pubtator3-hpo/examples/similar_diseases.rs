use pubtator3::{EntityId, Page};
use pubtator3_hpo::{Client, Dataset, SimilarityOptions};
use std::num::NonZeroUsize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut positional = Vec::new();
    let mut options = SimilarityOptions::default();
    let mut selected: Option<EntityId> = None;
    let mut papers = false;
    let mut allow_names = true;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--top" => {
                options.limit = args
                    .next()
                    .ok_or("missing --top value")?
                    .parse::<NonZeroUsize>()?
            }
            "--min-score" => {
                options.min_score = args.next().ok_or("missing --min-score value")?.parse()?
            }
            "--min-phenotypes" => {
                options.min_phenotypes = args
                    .next()
                    .ok_or("missing --min-phenotypes value")?
                    .parse()?
            }
            "--entity" => selected = Some(args.next().ok_or("missing --entity value")?.parse()?),
            "--papers" => papers = true,
            "--strict-ids" => allow_names = false,
            "--help" | "-h" => {
                println!(
                    "Usage: similar_diseases DATA_DIR [DISEASE_NAME] [--entity PUBTATOR_ID] [--top N] [--min-score 0..1] [--min-phenotypes N] [--strict-ids] [--papers]"
                );
                return Ok(());
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option {arg}").into()),
            _ => positional.push(arg),
        }
    }
    if positional.is_empty() || positional.len() > 2 {
        return Err("expected DATA_DIR and optional disease name; see --help".into());
    }
    let path = positional.remove(0);
    let name = positional
        .first()
        .map(String::as_str)
        .unwrap_or("Huntington disease");
    eprintln!("Loading HPO/Mondo snapshot...");
    let dataset = tokio::task::spawn_blocking(move || Dataset::from_dir(path)).await??;
    let client = Client::new(pubtator3::Client::new()?, dataset).allow_name_fallback(allow_names);
    let report = client
        .similar_diseases(name, selected.as_ref(), &options)
        .await?;
    println!(
        "{} → {} ({:?}, {:?})",
        report.entity.id, report.source.id, report.mapping.method, report.mapping.matched_value
    );
    println!(
        "{} positive source phenotypes; {} canonical diseases; HPO {}",
        report.source.phenotypes.len(),
        report.corpus_diseases,
        client.dataset().hpo_version
    );
    println!("Scores measure weighted phenotype overlap, not diagnostic probability or causality.");
    for disease in &report.matches {
        println!(
            "\n{:.4}\t{}\t{} ({} phenotypes)",
            disease.score, disease.disease_id, disease.name, disease.phenotype_count
        );
        println!(
            "  Exact shared: {}",
            disease
                .shared_phenotypes
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  Source-only terms: {}; direct conflicts: {}",
            disease.source_only_phenotypes.len(),
            disease.conflicting_phenotypes.len()
        );
        if papers {
            let targets = client.pubtator_entities(disease).await?;
            if targets.len() != 1 {
                println!(
                    "  PubTator mapping: {} candidates; no automatic paper lookup",
                    targets.len()
                );
                continue;
            }
            println!(
                "  PubTator: {} ({:?})",
                targets[0].entity.id, targets[0].method
            );
            let results = client
                .supporting_papers(&report.entity.id, &targets[0].entity.id, Page::FIRST)
                .await?;
            println!(
                "  {} co-mention papers (not typed disease–disease relations)",
                results.count
            );
            for paper in results.results.iter().take(3) {
                println!("    {} — {}", paper.pmid, paper.title);
            }
        }
    }
    Ok(())
}
