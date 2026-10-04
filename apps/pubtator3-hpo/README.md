# pubtator3-hpo

An async PubTator3 wrapper that maps diseases to curated HPO phenotype profiles and
finds diseases with similar profiles. The companion `pubtator3` crate still handles
the NCBI API. This crate supplies the disease mapping and local similarity search.

[biomedical_graph](../biomedical_graph/README.md) supplies Neo4j-backed caching
wrappers for both clients and persists their objects, ontology edges and evidence.

The identifier path is:

```text
PubTator disease → MeSH D…/C… → exact Mondo mapping → OMIM/ORPHA disease records → HP:… features
```

An `HP:` ID identifies a phenotype, not a disease. One disease usually has many HPO
features. Similarity searches the entire loaded HPO disease corpus, including
diseases that cannot be mapped back into PubTator.

## Run it

From the workspace root, download the datasets once into an empty directory:

```sh
cargo run -p pubtator3-hpo --example download_data -- /tmp/pubtator3-hpo-data

# List the curated phenotype profile and its mapping provenance.
cargo run -p pubtator3-hpo --example disease_phenotypes -- \
  /tmp/pubtator3-hpo-data 'Huntington disease'

# Compare the whole phenotype profile against other diseases.
cargo run -p pubtator3-hpo --example similar_diseases -- \
  /tmp/pubtator3-hpo-data 'Huntington disease' --top 10

cargo run -p pubtator3-hpo --example similar_diseases -- \
  /tmp/pubtator3-hpo-data CANVAS --top 10

# Optionally find PubTator entities for the results and fetch co-mention papers.
cargo run -p pubtator3-hpo --example similar_diseases -- \
  /tmp/pubtator3-hpo-data 'Huntington disease' --top 3 --papers

# Find diseases annotated with a symptom (including more specific HPO terms).
cargo run -p pubtator3-hpo --example symptom_diseases -- \
  /tmp/pubtator3-hpo-data Chorea --top 20

# Require both symptoms, or supply typed HPO IDs directly.
cargo run -p pubtator3-hpo --example symptom_diseases -- \
  /tmp/pubtator3-hpo-data Ataxia 'Peripheral neuropathy' --top 20
cargo run -p pubtator3-hpo --example symptom_diseases -- \
  /tmp/pubtator3-hpo-data HP:0002072 --exact --top 20
```

The download is roughly 150 MB. It retrieves `hp.obo` and `phenotype.hpoa` from the
same official HPO release, plus the official `mondo.json` release. Downloads are
staged and parsed before installation. `manifest.json` records release tags,
asset URLs, sizes, and SHA-256 hashes; it is installed last. Existing directories
containing files are rejected. Reuse the snapshot for reproducible results, and
download into a new directory when you want to update. `Dataset::from_dir` also
accepts manually supplied release files with those three filenames; it does not
check a manifest's hashes on every load. Dataset loading and ranking run locally.

An ambiguous query such as `Huntington` requires selecting the PubTator entity:

```sh
cargo run -p pubtator3-hpo --example similar_diseases -- \
  /tmp/pubtator3-hpo-data Huntington --entity '@DISEASE_Huntington_Disease'
```

Other filters: `--min-score 0.2`, `--min-phenotypes 5`, `--top 20`, and
`--strict-ids` to disable lexical mapping. Run `--help` for the full syntax.
The `--entity` value is a PubTator accession; `D006816` belongs in the mapper's
`MeshId` API instead.

`symptom_diseases` runs locally against HPO annotations. It accepts phenotype labels
or `HP:` IDs, normalizes case/whitespace, and prefers exact labels. If there is no
exact label, it searches label substrings and prints ambiguous candidates for you
to select by ID. It does not resolve symptom synonyms or silently choose between
multiple HPO concepts. Alternative and uniquely replaced obsolete IDs normalize
to their active term. For code, use `Dataset::find_phenotypes`, `phenotype`, and
`phenotype_is_a`.

By default, **all** supplied symptoms must match positive disease annotations;
more specific descendants count (e.g. a gait-ataxia annotation can match Ataxia).
`--any` accepts diseases matching at least one input and prioritizes the number of
matched symptoms. `--exact` disables descendant matching. Remaining ties sort by
disease name, then ID. Each result shows the actual matching HPO annotations and
its disease-record IDs. Absent-only features do not match; contradictory merged
annotations are flagged. This is a curated annotation lookup, with no diagnostic
score. Missing annotations do not establish symptom absence.

## Use the client

```toml
[dependencies]
pubtator3-hpo = { path = "../pubtator3-hpo" }
pubtator3 = { path = "../pubtator3" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use pubtator3_hpo::{Client, Dataset, SimilarityOptions};

#[tokio::main]
async fn main() -> pubtator3_hpo::Result<()> {
    // Parsing is CPU work: move it off the async runtime.
    let dataset = tokio::task::spawn_blocking(|| Dataset::from_dir("./phenotype-data"))
        .await??;
    let client = Client::new(pubtator3::Client::new()?, dataset);

    let profile = client.disease_phenotypes("Huntington disease", None).await?;
    println!("{:?}: {}", profile.mapping.method, profile.profile.id);
    for feature in &profile.profile.phenotypes {
        println!("{}: {}", feature.id, feature.name);
    }

    let options = SimilarityOptions { min_score: 0.2, ..Default::default() };
    let report = client.similar_diseases("Huntington disease", None, &options).await?;
    for disease in report.matches {
        println!("{:.3}: {} — {}", disease.score, disease.disease_id, disease.name);
        println!("Exact shared features: {:?}", disease.shared_phenotypes);
    }
    Ok(())
}
```

Reuse the client: clones share the `Arc<Dataset>` and the PubTator connection pool
and request pacing. Similarity computation runs through `spawn_blocking`; there
is one disease autocomplete request, rather than one request per ranked disease.
Only an explicit `pubtator_entities` or `supporting_papers` call adds literature
requests. `pubtator_entities` tries the canonical label and up to four short exact
Mondo synonyms, prefers MeSH matches, and returns all matching candidates found
before that stopping point. Reverse lookup is not exhaustive and can return
zero or multiple candidates. Inspect them before selecting a paper query.
`supporting_papers` returns the original paginated PubTator response.

## Offline mapping and ranking

You can skip PubTator entirely when you already know a disease ID:

```rust
use pubtator3_hpo::{Dataset, DiseaseId, MeshId, SimilarityOptions};

fn main() -> pubtator3_hpo::Result<()> {
    let dataset = Dataset::from_dir("./phenotype-data")?;
    let mesh: MeshId = "D006816".parse()?;
    let mapping = dataset.mapper().map_mesh(&mesh).require_unique()?;
    println!("{:?}", mapping);

    let omim: DiseaseId = "OMIM:143100".parse()?;
    let profile = dataset.profile(&omim).expect("Huntington annotations");
    println!("{} features", profile.phenotypes.len());
    let related = dataset.similar(&omim, &SimilarityOptions::default())?;
    println!("{} matches", related.len());
    Ok(())
}
```

`MeshId` supports descriptor and supplementary concept IDs. `MondoId`, `HpoId`,
and `DiseaseId` validate their namespace and numeric shape, including during
Serde deserialization. `DiseaseId` accepts `OMIM:`, `ORPHA:`, `DECIPHER:`, and
`MONDO:`, normalizing the namespace aliases `MIM:` and `Orphanet:`.

## Mapping rules

- Prefer **exact MeSH equivalence** in Mondo. Plain cross-references, related
  synonyms, and broader/narrower mappings are not equivalence evidence.
- If that link is absent, use an exact normalized disease label or EXACT synonym
  from Mondo/HPO. Normalization changes case and whitespace only. The mapping
  reports `ExactName`, so callers can distinguish lexical evidence from ID links.
- `allow_name_fallback(false)` disables the automatic lexical fallback. A known
  exact ID mapping without HPO records stays unmapped, even if a label matches.
- Return explicit `Mapped`, `Ambiguous`, or `Unmapped` outcomes. The wrapper
  rejects ambiguities rather than choosing the first result.
- Merge OMIM/Orphanet records only if each has a unique exact Mondo mapping.
  This avoids duplicate rankings and matches against another ID for the source.
  Records without such a mapping remain separate profiles.

Coverage gaps are real. With the September 2026 releases, Huntington's `D006816`
maps exactly to `MONDO:0007739`. PubTator's CANVAS identifier `C000726747` has no
exact link in that Mondo release. The query `CANVAS` works through Mondo's exact
synonym for `MONDO:0044720`, and the result reports `ExactName`. The longer query
`CANVAS syndrome` may require an explicit override because it is not that exact
synonym. You can provide a reviewed override for this case:

```rust
use pubtator3_hpo::{Client, Dataset};

fn main() -> pubtator3_hpo::Result<()> {
    let dataset = Dataset::from_dir("./phenotype-data")?;
    let client = Client::new(pubtator3::Client::new()?, dataset)
        .with_override("C000726747".parse()?, "OMIM:614575".parse()?)?;
    let mapping = client.mapper().map_mesh(&"C000726747".parse()?).require_unique()?;
    assert_eq!(mapping.method, pubtator3_hpo::MappingMethod::Override);
    Ok(())
}
```

Overrides also work with `dataset.mapper().with_override(...)`; the target must
exist in the loaded HPO data. The non-exact CANVAS cross-reference `OMIM:608088`
is deliberately excluded from its merged profile.

## What the score means

The score is **simGIC**, information-content-weighted Jaccard similarity of the
positive phenotype sets expanded with their HPO ancestors:

```text
IC(t) = -ln(number of canonical disease profiles containing t / corpus size)
score(A, B) = sum(IC(t), t in A ∩ B) / sum(IC(t), t in A ∪ B)
```

Each disease counts once. Common terms receive less weight; universal terms have
zero weight. Sibling terms can match through a meaningful shared ancestor, even
when `shared_phenotypes` has no exact term in common. Rankings are deterministic,
exclude the source and its exact-equivalent records, and omit zero similarities.
A profile with no positive phenotypes or no informative terms returns an error.

Only the Phenotypic abnormality branch (`HP:0000118`) and `P` annotation aspect
contribute. Alternative IDs and uniquely replaced obsolete IDs are normalized.
Unknown/unresolved terms and other aspects are counted as `unscored_annotations`.
`NOT` annotations, the HPO Excluded frequency class, `0%`, and `0/n` are excluded
from positive scoring. Profiles expose absent features and within-profile
contradictions. Matches expose direct present/absent conflicts in either direction;
these are reported without a score penalty. `source_only_phenotypes` means
unshared exact annotations, **not** known absence in the other disease.

Annotations retain their disease record ID, reference, evidence, onset, frequency,
sex, modifier, and biocuration metadata. Onset and positive feature frequencies
are currently not used as weights. Differences in annotation coverage and the
union of equivalent disease records affect rankings. The score is not a diagnosis,
a probability, or a PubTator disease–disease relationship. Paper lookup uses
co-mention (`disease A AND disease B`), which is a separate source of evidence.

## Validation and data sources

```sh
cargo test -p pubtator3-hpo
cargo clippy -p pubtator3-hpo --all-targets -- -D warnings
```

Tests cover mapping provenance, ambiguous/missing mappings, equivalent records,
negative annotations, obsolete/alternative HPO IDs, semantic ranking, graph
validation, PubTator query encoding, and wrapper HTTP behavior with a local server.
Synthetic fixtures are explicitly labeled. `mondo-real.json` retains the Huntington
and CANVAS nodes from the official `v2026-09-01` release. Live examples were checked
against that HPO/Mondo release pair on 2026-10-04.

Sources: [HPO annotation format](https://obophenotype.github.io/human-phenotype-ontology/annotations/phenotype_hpoa/),
[HPO releases](https://github.com/obophenotype/human-phenotype-ontology/releases),
[Mondo mapping guidance](https://mondo.monarchinitiative.org/pages/faq/),
[Mondo releases](https://github.com/monarch-initiative/mondo/releases), and
[PubTator3 API](https://www.ncbi.nlm.nih.gov/research/pubtator3/api).
