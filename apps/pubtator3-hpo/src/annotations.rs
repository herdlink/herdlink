use crate::{DiseaseId, Error, HpoId, PhenotypeAnnotation, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read};

#[derive(Deserialize)]
struct Row {
    database_id: String,
    disease_name: String,
    qualifier: String,
    hpo_id: String,
    reference: String,
    evidence: String,
    onset: String,
    frequency: String,
    sex: String,
    modifier: String,
    aspect: String,
    biocuration: String,
}
pub(crate) struct Record {
    pub name: String,
    pub annotations: Vec<PhenotypeAnnotation>,
}
pub(crate) struct Records {
    pub diseases: BTreeMap<DiseaseId, Record>,
    pub skipped_namespaces: usize,
}
fn nonempty(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}

pub(crate) fn read(reader: impl Read) -> Result<Records> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .quoting(false)
        .from_reader(reader);
    let mut records = Records {
        diseases: BTreeMap::new(),
        skipped_namespaces: 0,
    };
    for row in reader.deserialize::<Row>() {
        let row = row?;
        let id: DiseaseId = match row.database_id.parse() {
            Ok(id) => id,
            Err(_)
                if ![
                    "OMIM:",
                    "MIM:",
                    "ORPHA:",
                    "Orphanet:",
                    "MONDO:",
                    "DECIPHER:",
                ]
                .iter()
                .any(|p| row.database_id.starts_with(p)) =>
            {
                records.skipped_namespaces += 1;
                continue;
            }
            Err(e) => return Err(e),
        };
        let phenotype: HpoId = row.hpo_id.parse()?;
        if !matches!(row.qualifier.as_str(), "" | "NOT") {
            return Err(Error::Data(format!(
                "unknown HPO qualifier {:?}",
                row.qualifier
            )));
        }
        let excluded = row.qualifier == "NOT" || zero_frequency(&row.frequency);
        let record = records
            .diseases
            .entry(id.clone())
            .or_insert_with(|| Record {
                name: row.disease_name.clone(),
                annotations: Vec::new(),
            });
        record.annotations.push(PhenotypeAnnotation {
            disease_id: id,
            phenotype,
            excluded,
            reference: row.reference,
            evidence: row.evidence,
            onset: nonempty(row.onset),
            frequency: nonempty(row.frequency),
            sex: nonempty(row.sex),
            modifier: nonempty(row.modifier),
            aspect: row.aspect,
            biocuration: row.biocuration,
        });
    }
    if records.diseases.is_empty() {
        return Err(Error::Data(
            "HPO annotations contain no disease records".into(),
        ));
    }
    Ok(records)
}

fn zero_frequency(s: &str) -> bool {
    if s == "HP:0040285" {
        return true;
    } // Excluded
    if let Some(p) = s.strip_suffix('%') {
        return p.parse::<f64>().is_ok_and(|p| p == 0.0);
    }
    if let Some((n, d)) = s.split_once('/') {
        return n.parse::<u64>().is_ok_and(|n| n == 0) && d.parse::<u64>().is_ok_and(|d| d > 0);
    }
    false
}
