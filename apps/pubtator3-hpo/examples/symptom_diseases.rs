use pubtator3_hpo::{Dataset, DiseaseProfile, Phenotype};
use std::{collections::BTreeSet, num::NonZeroUsize};

#[derive(Debug)]
struct Options {
    directory: String,
    symptoms: Vec<String>,
    top: NonZeroUsize,
    any: bool,
    exact: bool,
}

fn arguments(
    args: impl IntoIterator<Item = String>,
) -> Result<Option<Options>, Box<dyn std::error::Error>> {
    let mut args = args.into_iter();
    let mut positional = Vec::new();
    let mut top = NonZeroUsize::new(20).unwrap();
    let mut any = false;
    let mut exact = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--top" => top = args.next().ok_or("missing --top value")?.parse()?,
            "--any" => any = true,
            "--exact" => exact = true,
            "--help" | "-h" => {
                println!(
                    "Usage: symptom_diseases DATA_DIR SYMPTOM_OR_HPO_ID... [--top N] [--any] [--exact]"
                );
                println!(
                    "Examples: DATA_DIR Chorea; DATA_DIR Ataxia 'Peripheral neuropathy'; DATA_DIR HP:0002072"
                );
                return Ok(None);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option {arg}").into()),
            _ => positional.push(arg),
        }
    }
    if positional.len() < 2 {
        return Err("expected DATA_DIR and at least one symptom name or HPO ID; see --help".into());
    }
    Ok(Some(Options {
        directory: positional.remove(0),
        symptoms: positional,
        top,
        any,
        exact,
    }))
}

struct Match<'a> {
    disease: &'a DiseaseProfile,
    // Retain the actual annotations so broader queries explain descendant matches.
    symptoms: Vec<(&'a Phenotype, Vec<&'a Phenotype>)>,
}

fn matches<'a>(
    dataset: &'a Dataset,
    symptoms: &'a [Phenotype],
    any: bool,
    exact: bool,
) -> Vec<Match<'a>> {
    let mut matches = Vec::new();
    for disease in dataset.profiles() {
        let matched: Vec<_> = symptoms
            .iter()
            .filter_map(|symptom| {
                let features: Vec<_> = disease
                    .phenotypes
                    .iter()
                    .filter(|feature| {
                        if exact {
                            feature.id == symptom.id
                        } else {
                            dataset.phenotype_is_a(&feature.id, &symptom.id)
                        }
                    })
                    .collect();
                (!features.is_empty()).then_some((symptom, features))
            })
            .collect();
        if !matched.is_empty() && (any || matched.len() == symptoms.len()) {
            matches.push(Match {
                disease,
                symptoms: matched,
            });
        }
    }
    // This is annotation lookup, not diagnosis scoring. Any-mode prioritizes more
    // matched inputs; ties are alphabetical and deterministic.
    matches.sort_by(|a, b| {
        b.symptoms
            .len()
            .cmp(&a.symptoms.len())
            .then_with(|| {
                a.disease
                    .name
                    .to_lowercase()
                    .cmp(&b.disease.name.to_lowercase())
            })
            .then_with(|| a.disease.id.cmp(&b.disease.id))
    });
    matches
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(options) = arguments(std::env::args().skip(1))? else {
        return Ok(());
    };
    eprintln!("Loading HPO/Mondo snapshot...");
    let dataset = Dataset::from_dir(&options.directory)?;
    let mut symptoms = Vec::new();
    let mut seen = BTreeSet::new();
    for query in &options.symptoms {
        let mut candidates = dataset.find_phenotypes(query);
        if candidates.is_empty() {
            return Err(format!(
                "no HPO phenotype label/ID matching {query:?}; use an HPO label or HP:… ID"
            )
            .into());
        }
        if candidates.len() != 1 {
            eprintln!("{} HPO candidates for {query:?}:", candidates.len());
            for phenotype in &candidates {
                eprintln!("  {} — {}", phenotype.id, phenotype.name);
            }
            return Err("ambiguous symptom; replace it with the intended HP:… ID".into());
        }
        let phenotype = candidates.remove(0);
        if seen.insert(phenotype.id.clone()) {
            symptoms.push(phenotype);
        }
    }
    for symptom in &symptoms {
        println!("Symptom: {} — {}", symptom.id, symptom.name);
    }
    println!(
        "Matching {} symptoms using {} annotations; HPO {}",
        if options.any {
            "any supplied"
        } else {
            "all supplied"
        },
        if options.exact {
            "exact"
        } else {
            "exact or descendant"
        },
        dataset.hpo_version
    );
    let results = matches(&dataset, &symptoms, options.any, options.exact);
    println!(
        "{} diseases match; showing up to {}.",
        results.len(),
        options.top
    );
    println!("Ordered by matched symptom count, then name; these are curated annotation matches.");
    for result in results.iter().take(options.top.get()) {
        println!(
            "\n{}\t{}\t{}/{} symptoms",
            result.disease.id,
            result.disease.name,
            result.symptoms.len(),
            symptoms.len()
        );
        println!(
            "  Annotation records: {}",
            result
                .disease
                .annotation_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        for (symptom, features) in &result.symptoms {
            println!(
                "  {} {} ← {}",
                symptom.id,
                symptom.name,
                features
                    .iter()
                    .map(|feature| format!("{} {}", feature.id, feature.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            // A merged profile can contain contradictory positive and NOT records.
            let conflicts: Vec<_> = features
                .iter()
                .filter(|feature| {
                    result
                        .disease
                        .conflicting_phenotypes
                        .iter()
                        .any(|conflict| conflict.id == feature.id)
                })
                .collect();
            if !conflicts.is_empty() {
                println!("    Present/absent conflict in annotation records");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn dataset() -> Dataset {
        Dataset::from_readers(
            Cursor::new(include_str!("../tests/fixtures/hp.obo")),
            include_bytes!("../tests/fixtures/phenotype.hpoa").as_slice(),
            include_bytes!("../tests/fixtures/mondo.json").as_slice(),
        )
        .unwrap()
    }

    #[test]
    fn exact_and_descendant_queries_match_only_positive_annotations() {
        let ds = dataset();
        let symptoms = [ds.phenotype(&"HP:0001001".parse().unwrap()).unwrap()];
        assert!(matches(&ds, &symptoms, false, true).is_empty());
        let result = matches(&ds, &symptoms, false, false);
        assert_eq!(result.len(), 4);
        assert!(result.iter().any(|r| r.disease.name == "Sibling Disease"));
        let symptoms = [ds.phenotype(&"HP:0001003".parse().unwrap()).unwrap()];
        let result = matches(&ds, &symptoms, false, true);
        assert_eq!(result.len(), 1); // Source's NOT/0% annotations must not match.
        assert_eq!(result[0].disease.name, "Sibling Disease");
    }

    #[test]
    fn multiple_symptoms_support_all_and_any_with_explanations() {
        let ds = dataset();
        let symptoms = [
            ds.phenotype(&"HP:0001002".parse().unwrap()).unwrap(),
            ds.phenotype(&"HP:0001004".parse().unwrap()).unwrap(),
        ];
        let all = matches(&ds, &symptoms, false, true);
        assert_eq!(all.len(), 2); // Source and Twin, not duplicate OMIM/ORPHA entries.
        assert!(all.iter().all(|m| m.symptoms.len() == 2));
        let any = matches(&ds, &symptoms, true, true);
        assert_eq!(any.len(), 3);
        assert_eq!(any.last().unwrap().disease.name, "Partial Disease");
        assert_eq!(
            any.last().unwrap().symptoms[0].1[0].id.as_str(),
            "HP:0001002"
        );
    }
}
