use std::{
    collections::BTreeMap,
    num::{NonZeroU32, NonZeroUsize},
};

use pubtator3::{
    AutocompleteRequest, Client, Concept, EntityId, RelatedEntity, RelationEntityType,
    RelationType, RelationsRequest,
};

/// List every distinct gene returned by PubTator's disease-relation endpoint.
/// The endpoint has no total count or documented pagination; completeness is unknown.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut positional = Vec::new();
    let mut association_only = false;
    let mut genes_only = false;
    let mut min_publications = 0_u64;
    let mut top = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--associate" => association_only = true,
            "--genes-only" => genes_only = true,
            "--min-publications" => {
                min_publications = args
                    .next()
                    .ok_or("missing value for --min-publications")?
                    .parse()
                    .map_err(|_| "--min-publications must be a nonnegative integer")?;
            }
            "--top" => {
                top = Some(
                    args.next()
                        .ok_or("missing value for --top")?
                        .parse::<NonZeroUsize>()
                        .map_err(|_| "--top must be a positive integer")?,
                );
            }
            "--help" | "-h" => {
                println!(
                    "Usage: cargo run -p pubtator3 --example disease_genes -- [DISEASE_NAME] [ENTITY_ID] [--associate] [--genes-only] [--min-publications N] [--top N]"
                );
                println!("Default disease: Henoch Schonlein purpura; default: all relation types.");
                println!(
                    "Genes are ranked by their strongest relation's publication count. These counts measure literature support, not established causality."
                );
                return Ok(());
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}").into()),
            _ => positional.push(arg),
        }
    }
    if positional.len() > 2 {
        return Err("expected a disease name and optional entity ID".into());
    }
    let name = positional
        .first()
        .map(String::as_str)
        .unwrap_or("Henoch Schonlein purpura");
    let selected: Option<EntityId> = positional.get(1).map(|id| id.parse()).transpose()?;
    let client = Client::new()?;
    let candidates = client
        .autocomplete(&AutocompleteRequest::new(name).concept(Concept::Disease))
        .await?;
    for disease in &candidates {
        eprintln!(
            "Candidate: {} — {} ({:?})",
            disease.id, disease.name, disease.db_id
        );
    }
    let disease = match selected {
        Some(id) => candidates
            .iter()
            .find(|disease| disease.id == id)
            .ok_or("selected entity ID was not returned by autocomplete")?,
        None if candidates.len() == 1 => &candidates[0],
        None if candidates.is_empty() => {
            return Err("no disease match; try another name or spelling".into());
        }
        None => {
            return Err(
                "multiple candidates; pass the intended entity ID as the second argument".into(),
            );
        }
    };

    let mut limit = 1_000_u32;
    let relations = loop {
        let mut request = RelationsRequest::new(disease.id.clone())
            .target_type(RelationEntityType::Gene)
            .limit(NonZeroU32::new(limit).unwrap());
        if association_only {
            request = request.relation_type(RelationType::Associate);
        }
        let result = client.relations(&request).await?;
        eprintln!(
            "Retrieved {} relation records with limit {limit}.",
            result.len()
        );
        if result.len() < limit as usize {
            break result;
        }
        limit = limit.checked_mul(10).ok_or(
            "response still fills the maximum limit; use the PubTator bulk relations data",
        )?;
    };

    let mut genes: BTreeMap<EntityId, Vec<RelatedEntity>> = BTreeMap::new();
    for relation in relations {
        // A gene can appear on either side of a relation. Validate both the disease
        // endpoint and the gene namespace instead of assuming the target is a gene.
        let gene = if relation.source == disease.id && relation.target.namespace() == "GENE" {
            relation.target.clone()
        } else if relation.target == disease.id && relation.source.namespace() == "GENE" {
            relation.source.clone()
        } else {
            return Err(format!("unexpected disease–gene relation: {relation:?}").into());
        };
        genes.entry(gene).or_default().push(relation);
    }

    let available_genes = genes.len();
    let genes = select_genes(genes, min_publications, top);
    eprintln!("Disease: {} [{}]", disease.name, disease.id);
    eprintln!(
        "{} of {available_genes} distinct genes in {} qualifying relation records ({}).",
        genes.len(),
        genes
            .iter()
            .map(|(_, relations)| relations.len())
            .sum::<usize>(),
        if association_only {
            "associate only"
        } else {
            "all relation types"
        }
    );
    eprintln!(
        "Minimum publications per relation: {min_publications}; ranked by strongest relation count."
    );
    eprintln!(
        "No total count or pagination is supplied by this endpoint; an exhaustive list cannot be verified."
    );
    eprintln!(
        "Publication counts are per relation and can overlap; they are not summed across relation types."
    );
    if min_publications > 0 || top.is_some() {
        eprintln!(
            "Filtering by publication support is a popularity heuristic; it does not establish causality."
        );
    }
    // TSV on stdout can be saved or imported into another tool. Diagnostics go to stderr.
    if genes_only {
        for (gene, _) in &genes {
            println!("{gene}");
        }
    } else {
        println!("gene_id\trelation\tsource\ttarget\tpublications");
        for (gene, relations) in genes {
            for relation in relations {
                println!(
                    "{gene}\t{}\t{}\t{}\t{}",
                    relation.relation_type, relation.source, relation.target, relation.publications
                );
            }
        }
    }
    Ok(())
}

fn select_genes(
    mut genes: BTreeMap<EntityId, Vec<RelatedEntity>>,
    min_publications: u64,
    top: Option<NonZeroUsize>,
) -> Vec<(EntityId, Vec<RelatedEntity>)> {
    for relations in genes.values_mut() {
        relations.retain(|relation| relation.publications >= min_publications);
    }
    let mut ranked: Vec<_> = genes
        .into_iter()
        .filter(|(_, relations)| !relations.is_empty())
        .collect();
    let support = |relations: &[RelatedEntity]| {
        relations
            .iter()
            .map(|relation| relation.publications)
            .max()
            .unwrap_or(0)
    };
    ranked.sort_by(|(a, a_relations), (b, b_relations)| {
        support(b_relations)
            .cmp(&support(a_relations))
            .then_with(|| a.cmp(b))
    });
    if let Some(top) = top {
        ranked.truncate(top.get());
    }
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(gene: &str, kind: RelationType, publications: u64) -> RelatedEntity {
        RelatedEntity {
            source: "@DISEASE_Huntington_Disease".parse().unwrap(),
            target: gene.parse().unwrap(),
            relation_type: kind,
            publications,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn overlapping_support_is_not_added_and_top_limits_distinct_genes() {
        let genes = BTreeMap::from([
            (
                "@GENE_A".parse().unwrap(),
                vec![
                    edge("@GENE_A", RelationType::Associate, 60),
                    edge("@GENE_A", RelationType::Stimulate, 60),
                ],
            ),
            (
                "@GENE_B".parse().unwrap(),
                vec![edge("@GENE_B", RelationType::Associate, 90)],
            ),
        ]);
        let top = select_genes(genes.clone(), 0, NonZeroUsize::new(1));
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0.as_str(), "@GENE_B");
        let filtered = select_genes(genes.clone(), 61, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0.as_str(), "@GENE_B");
        assert_eq!(select_genes(genes, 60, None).len(), 2);
    }
}
