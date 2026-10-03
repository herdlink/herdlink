use pubtator3_hpo::{Client, Dataset};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("Usage: disease_phenotypes DATA_DIR [DISEASE_NAME] [ENTITY_ID]")?;
    let name = args.next().unwrap_or_else(|| "Huntington disease".into());
    let selected = args.next().map(|id| id.parse()).transpose()?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let dataset = tokio::task::spawn_blocking(move || Dataset::from_dir(path)).await??;
    let client = Client::new(pubtator3::Client::new()?, dataset);
    let result = client.disease_phenotypes(&name, selected.as_ref()).await?;
    println!(
        "{} → {} ({:?}, matched {:?})",
        result.entity.id, result.profile.id, result.mapping.method, result.mapping.matched_value
    );
    println!("Annotation records: {:?}", result.mapping.annotation_ids);
    println!("Positive phenotypes: {}", result.profile.phenotypes.len());
    for p in &result.profile.phenotypes {
        println!("{}\t{}", p.id, p.name);
    }
    println!(
        "Explicitly absent: {}",
        result.profile.excluded_phenotypes.len()
    );
    println!(
        "Conflicting: {}; unscored annotation rows: {}",
        result.profile.conflicting_phenotypes.len(),
        result.profile.unscored_annotations
    );
    Ok(())
}
