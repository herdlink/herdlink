use pubtator3::{Client, ExportFormat, Pmid, TextScope};

/// Print a raw export for a downstream BioC or PubTator pipeline.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pmid: Pmid = args.next().unwrap_or_else(|| "19894120".into()).parse()?;
    let format = match args.next().as_deref().unwrap_or("pubtator") {
        "pubtator" => ExportFormat::PubTator,
        "biocxml" => ExportFormat::BioCXml,
        "biocjson" => ExportFormat::BioCJson,
        _ => return Err("format must be pubtator, biocxml, or biocjson".into()),
    };
    print!(
        "{}",
        Client::new()?
            .export(&[pmid], format, TextScope::Abstract)
            .await?
    );
    Ok(())
}
