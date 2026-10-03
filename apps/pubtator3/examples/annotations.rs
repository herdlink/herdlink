use pubtator3::{Client, Pmcid, Pmid, TextScope};

/// Inspect abstract entities/relations and fetch the associated PMC full text.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    let pmids = [Pmid::new(19894120)?, Pmid::new(29355051)?];
    for document in client
        .annotations_batched(&pmids, TextScope::Abstract)
        .await?
    {
        println!("Document {} (PMID {:?})", document.id, document.pmid);
        for annotation in document.annotations() {
            println!(
                "  {}: {:?}, {:?}",
                annotation.text, annotation.infons.accession, annotation.locations
            );
        }
        for relation in document.all_relations() {
            println!(
                "  Relation {:?}: {:?} -> {:?}",
                relation.infons.relation_type,
                relation
                    .infons
                    .role1
                    .as_ref()
                    .and_then(|r| r.accession.as_ref()),
                relation
                    .infons
                    .role2
                    .as_ref()
                    .and_then(|r| r.accession.as_ref())
            );
        }
    }
    let pmcid: Pmcid = "PMC6142073".parse()?;
    for document in client.pmc_annotations(&[pmcid]).await? {
        println!(
            "Full text {}: {} passages",
            document.id,
            document.passages.len()
        );
    }
    Ok(())
}
