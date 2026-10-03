use crate::{DiseaseId, MeshId, MondoId, Result};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};

#[derive(Debug, Clone)]
pub(crate) struct MondoDisease {
    pub name: String,
    pub names: Vec<String>,
    pub mesh: BTreeSet<MeshId>,
    pub diseases: BTreeSet<DiseaseId>,
}

#[derive(Debug, Default)]
pub(crate) struct MondoIndex {
    pub terms: BTreeMap<MondoId, MondoDisease>,
    pub disease_to_mondo: BTreeMap<DiseaseId, BTreeSet<MondoId>>,
    pub mesh_to_mondo: BTreeMap<MeshId, BTreeSet<MondoId>>,
}

#[derive(Deserialize)]
struct Document {
    graphs: Vec<Graph>,
}
#[derive(Deserialize)]
struct Graph {
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    id: String,
    #[serde(default)]
    lbl: String,
    #[serde(default)]
    meta: Meta,
}
#[derive(Default, Deserialize)]
struct Meta {
    #[serde(default)]
    deprecated: bool,
    #[serde(default)]
    synonyms: Vec<Property>,
    #[serde(default, rename = "basicPropertyValues")]
    properties: Vec<Property>,
    #[serde(default)]
    xrefs: Vec<Xref>,
}
#[derive(Deserialize)]
struct Property {
    pred: String,
    val: String,
}
#[derive(Deserialize)]
struct Xref {
    val: String,
    #[serde(default)]
    meta: Meta,
}

pub(crate) fn normalize_name(name: &str) -> String {
    // Conservative: case and whitespace only. No fuzzy matching or dropped words.
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn curie(uri: &str) -> Option<String> {
    for (prefix, namespace) in [
        ("http://identifiers.org/mesh/", "MESH:"),
        ("https://identifiers.org/mesh/", "MESH:"),
        ("http://id.nlm.nih.gov/mesh/", "MESH:"),
        ("https://id.nlm.nih.gov/mesh/", "MESH:"),
        ("https://omim.org/entry/", "OMIM:"),
        ("http://omim.org/entry/", "OMIM:"),
        ("http://www.orpha.net/ORDO/Orphanet_", "ORPHA:"),
        ("https://www.orpha.net/ORDO/Orphanet_", "ORPHA:"),
    ] {
        if let Some(id) = uri.strip_prefix(prefix) {
            return Some(format!("{namespace}{id}"));
        }
    }
    if let Some(id) = uri.strip_prefix("http://purl.obolibrary.org/obo/MONDO_") {
        return Some(format!("MONDO:{id}"));
    }
    None
}

impl MondoIndex {
    pub fn read(reader: impl Read) -> Result<Self> {
        let doc: Document = serde_json::from_reader(reader)?;
        let mut index = Self::default();
        for node in doc.graphs.into_iter().flat_map(|g| g.nodes) {
            if node.meta.deprecated {
                continue;
            }
            let Some(id) = curie(&node.id).and_then(|id| id.parse::<MondoId>().ok()) else {
                continue;
            };
            let mut term = MondoDisease {
                name: node.lbl.clone(),
                names: vec![node.lbl],
                mesh: BTreeSet::new(),
                diseases: BTreeSet::from([id.to_string().parse()?]),
            };
            term.names.extend(
                node.meta
                    .synonyms
                    .into_iter()
                    .filter(|s| s.pred.ends_with("hasExactSynonym"))
                    .map(|s| s.val),
            );
            let mut exact: BTreeSet<String> = node
                .meta
                .properties
                .into_iter()
                .filter(|p| p.pred.ends_with("#exactMatch"))
                .filter_map(|p| curie(&p.val))
                .collect();
            // Plain xrefs (and related/broad/narrow mappings) are not equivalence evidence.
            exact.extend(
                node.meta
                    .xrefs
                    .into_iter()
                    .filter(|x| {
                        x.meta
                            .properties
                            .iter()
                            .any(|p| p.val == "MONDO:equivalentTo")
                    })
                    .map(|x| x.val),
            );
            for xref in exact {
                if let Some(mesh) = xref
                    .strip_prefix("MESH:")
                    .and_then(|s| s.parse::<MeshId>().ok())
                {
                    term.mesh.insert(mesh);
                } else if let Ok(disease) = xref.parse::<DiseaseId>() {
                    term.diseases.insert(disease);
                }
            }
            for disease in &term.diseases {
                index
                    .disease_to_mondo
                    .entry(disease.clone())
                    .or_default()
                    .insert(id.clone());
            }
            for mesh in &term.mesh {
                index
                    .mesh_to_mondo
                    .entry(mesh.clone())
                    .or_default()
                    .insert(id.clone());
            }
            index.terms.insert(id, term);
        }
        if index.terms.is_empty() {
            return Err(crate::Error::Data(
                "Mondo contains no active diseases".into(),
            ));
        }
        Ok(index)
    }
}
