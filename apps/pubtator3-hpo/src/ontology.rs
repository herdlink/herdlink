use crate::{Error, HpoId, Result};
use hpo::{Ontology, builder::Builder};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::BufRead,
};

#[derive(Default)]
struct Term {
    id: Option<HpoId>,
    name: String,
    parents: Vec<HpoId>,
    aliases: Vec<HpoId>,
    obsolete: bool,
    replacements: Vec<HpoId>,
}

pub(crate) struct ParsedOntology {
    pub ontology: Ontology,
    pub aliases: BTreeMap<HpoId, HpoId>,
    pub version: String,
}

pub(crate) fn read(reader: impl BufRead) -> Result<ParsedOntology> {
    let mut terms = BTreeMap::new();
    let mut current = None;
    let mut version = String::new();
    let commit = |current: &mut Option<Term>, terms: &mut BTreeMap<HpoId, Term>| -> Result<()> {
        if let Some(mut term) = current.take() {
            let id = term
                .id
                .take()
                .ok_or_else(|| Error::Data("HPO term without ID".into()))?;
            if terms.insert(id.clone(), term).is_some() {
                return Err(Error::Data(format!("duplicate HPO term {id}")));
            }
        }
        Ok(())
    };
    for line in reader.lines() {
        let line = line?;
        if let Some(v) = line.strip_prefix("data-version: ") {
            version = v.into();
        }
        if line.starts_with('[') {
            commit(&mut current, &mut terms)?;
            current = (line == "[Term]").then(Term::default);
            continue;
        }
        let Some(term) = &mut current else {
            continue;
        };
        if let Some(id) = line.strip_prefix("id: ") {
            term.id = Some(id.parse()?);
        }
        if let Some(name) = line.strip_prefix("name: ") {
            term.name = name.into();
        }
        if let Some(id) = line.strip_prefix("is_a: ") {
            term.parents
                .push(id.split_whitespace().next().unwrap_or("").parse()?);
        }
        if let Some(id) = line.strip_prefix("alt_id: ") {
            term.aliases.push(id.parse()?);
        }
        if let Some(id) = line.strip_prefix("replaced_by: ") {
            term.replacements.push(id.parse()?);
        }
        if line == "is_obsolete: true" {
            term.obsolete = true;
        }
    }
    commit(&mut current, &mut terms)?;
    if !terms.contains_key(&"HP:0000118".parse()?) {
        return Err(Error::Data(
            "HPO lacks Phenotypic abnormality (HP:0000118)".into(),
        ));
    }
    let mut aliases = BTreeMap::new();
    let mut builder = Builder::new();
    for (id, term) in &terms {
        if term.obsolete {
            if term.replacements.len() == 1 {
                aliases.insert(id.clone(), term.replacements[0].clone());
            }
        } else {
            if term.name.is_empty() {
                return Err(Error::Data(format!("HPO term {id} has no label")));
            }
            builder.new_term(&term.name, id.number());
            for alias in &term.aliases {
                aliases.insert(alias.clone(), id.clone());
            }
        }
    }
    // Resolve obsolete replacement chains without silently picking between replacements.
    for start in aliases.keys().cloned().collect::<Vec<_>>() {
        let mut seen = BTreeSet::new();
        let mut id = start.clone();
        while let Some(next) = aliases.get(&id) {
            if !seen.insert(id.clone()) {
                return Err(Error::Data(format!("HPO replacement cycle at {id}")));
            }
            id = next.clone();
        }
        if terms.get(&id).is_some_and(|t| !t.obsolete) {
            aliases.insert(start, id);
        } else {
            aliases.remove(&start);
        }
    }
    // Validate the parent graph before handing it to hpo's builder, which assumes a DAG.
    fn visit(
        id: &HpoId,
        terms: &BTreeMap<HpoId, Term>,
        seen: &mut BTreeSet<HpoId>,
        active: &mut BTreeSet<HpoId>,
    ) -> Result<()> {
        if seen.contains(id) {
            return Ok(());
        }
        if !active.insert(id.clone()) {
            return Err(Error::Data(format!("HPO parent cycle at {id}")));
        }
        for parent in &terms[id].parents {
            if !terms.get(parent).is_some_and(|p| !p.obsolete) {
                return Err(Error::Data(format!("missing/obsolete HPO parent {parent}")));
            }
            visit(parent, terms, seen, active)?;
        }
        active.remove(id);
        seen.insert(id.clone());
        Ok(())
    }
    let mut seen = BTreeSet::new();
    for (id, term) in &terms {
        if !term.obsolete {
            visit(id, &terms, &mut seen, &mut BTreeSet::new())?;
        }
    }
    let mut builder = builder.terms_complete();
    for (id, term) in &terms {
        if !term.obsolete {
            for parent in &term.parents {
                builder.add_parent(parent.number(), id.number())?;
            }
        }
    }
    let ontology = builder
        .connect_all_terms()
        .calculate_information_content()?
        .build_minimal();
    Ok(ParsedOntology {
        ontology,
        aliases,
        version,
    })
}
