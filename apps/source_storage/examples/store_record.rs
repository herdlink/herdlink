use serde_json::json;
use source_storage::{
    Category, Classification, Client, Content, FileFormat, NewRecord, RecordStatus, Source,
};

fn main() -> source_storage::Result<()> {
    let client = Client::open("records.sqlite", "objects")?;
    let record = client.insert_with_file(
        NewRecord {
            file_format: FileFormat::Json,
            classification: Classification {
                category: Category::ClinicalStudy,
                subtype: "interventional_trial".into(),
                is_peer_reviewed: false,
            },
            source: Source {
                source_name: "clinicaltrials_gov".into(),
                external_id: "NCT00000000".into(),
                doi: None,
                source_url: "https://clinicaltrials.gov/study/NCT00000000".into(),
            },
            content: Content {
                title: "Example trial".into(),
                summary: None,
                language: "en".into(),
                authors: None,
                published_at: None,
                source_updated_at: None,
                record_status: RecordStatus::Active,
                metadata: json!({"phase": "PHASE2", "enrollment": 100}),
            },
        },
        br#"{"example":true}"#,
    )?;

    let loaded = client.get(record.storage.id)?.expect("inserted record");
    let bytes = client.read_file(loaded.storage.id)?;
    println!(
        "{}: {} bytes at {}",
        loaded.content.title,
        bytes.len(),
        loaded.storage.storage_uri
    );
    Ok(())
}
