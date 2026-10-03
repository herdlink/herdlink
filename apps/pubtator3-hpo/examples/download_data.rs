#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "phenotype-data".into());
    eprintln!("Downloading official HPO and Mondo releases into {path} (about 150 MB).");
    let manifest = pubtator3_hpo::download_data(&path).await?;
    println!(
        "HPO: {}; Mondo: {}",
        manifest.hpo_release, manifest.mondo_release
    );
    for file in manifest.files {
        println!(
            "{}: {} bytes, SHA-256 {}",
            file.name, file.bytes, file.sha256
        );
    }
    Ok(())
}
