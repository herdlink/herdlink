# pubtator3

Async Rust client for the [official PubTator3 API](https://www.ncbi.nlm.nih.gov/research/pubtator3/api).
Supports entity autocomplete, relation discovery, paginated literature search, abstract/full-text
annotation retrieval, raw PubTator/BioC JSON/BioC XML exports, and MeSH entry-term listing
through NLM's companion API. No API key is required.

For automatic disease-to-HPO mapping and phenotype-based similar disease search,
use the companion [pubtator3-hpo crate](../pubtator3-hpo/README.md).

Add this local crate to an app (adjust the relative path as needed):

```toml
[dependencies]
pubtator3 = { path = "../pubtator3" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use pubtator3::{Client, EntityId, Page, Pmid, SearchQuery, TextScope};

#[tokio::main]
async fn main() -> pubtator3::Result<()> {
    let client = Client::new()?;
    let gene: EntityId = "@GENE_BRAF".parse()?;
    let hits = client.search(&SearchQuery::entity(&gene), Page::FIRST).await?;
    for article in &hits.results {
        println!("{}: {}", article.pmid, article.title);
    }

    let pmid = Pmid::new(19894120)?;
    for document in client.annotations(&[pmid], TextScope::Abstract).await? {
        for annotation in document.annotations() {
            println!("{} -> {:?}", annotation.text, annotation.infons.accession);
        }
    }
    Ok(())
}
```

## Use cases

Run from the workspace root:

```sh
# Normalize a gene name and inspect two pages of papers.
cargo run -p pubtator3 --example entity_search -- BRAF

# Discover drug–disease treatment relations and supporting publications.
cargo run -p pubtator3 --example relations

# Extract normalized entities, relation participants, and PMC full text.
cargo run -p pubtator3 --example annotations

# Feed raw text/XML/JSON into another tool.
cargo run -p pubtator3 --example export -- 19894120 biocxml

# Normalize a disease synonym and list its MeSH entry terms.
cargo run -p pubtator3 --example synonyms -- "Lou Gehrig disease"

# Find extracted treatment/prevention relations and matching literature for IgA vasculitis.
cargo run -p pubtator3 --example disease_treatment_prevention

# List every related gene returned for IgA vasculitis, across all relation types.
cargo run -p pubtator3 --example disease_genes

# Print only the distinct gene IDs, one per line.
cargo run -p pubtator3 --example disease_genes -- --genes-only

# Restrict the list to association relations, or choose another disease.
cargo run -p pubtator3 --example disease_genes -- "Henoch Schonlein purpura" --associate
cargo run -p pubtator3 --example disease_genes -- "Lou Gehrig disease"
```

`disease_genes` groups all returned relation records by distinct `EntityId`, accepts
genes on either side of a relation, and preserves each relation's publication count.
It starts with a limit of 1,000 and retries with a larger limit if the response fills it.
The relation endpoint supplies no total count or documented pagination, so the example
cannot certify that the upstream result is exhaustive. For exhaustive dataset analysis,
use the [PubTator3 bulk relations downloads](https://ftp.ncbi.nlm.nih.gov/pub/lu/PubTator3/).
The optional second positional argument selects a disease entity ID when autocomplete
is ambiguous. TSV data goes to stdout and diagnostics go to stderr, so you can save it:

```sh
cargo run -p pubtator3 --example disease_genes > iga-vasculitis-genes.tsv
```

`--genes-only` writes one distinct gene ID per line instead of relation records.

For connections with more literature support, filter individual relation records
by their publication count and optionally select the top distinct genes:

```sh
cargo run -p pubtator3 --example disease_genes -- \
  "Huntington" "@DISEASE_Huntington_Disease" --associate --min-publications 1000 --genes-only

cargo run -p pubtator3 --example disease_genes -- \
  "Huntington" "@DISEASE_Huntington_Disease" --associate --top 5
```

These are client-side filters. `--min-publications N` retains relation records with
at least N supporting publications; `--top N` keeps N distinct genes after filtering.
Genes are ordered by their strongest remaining relation's publication count, with
gene ID as the tie-breaker. Counts are never summed across overlapping relation types.
The example has no default minimum or top-N cutoff. Thresholds measure literature
popularity and need to be chosen for the disease; they do not establish causality.

The relation-discovery API returns publication counts but has no documented
minimum-score parameter. `Publication::score` is a search-result ranking score,
while `RelationInfo::score` belongs to an individual extracted BioC relation.
Neither represents a curated confidence that a gene causes the disease. BioC scores
can be filtered after fetching annotations, but that does not provide a causal-gene query.

One gene may have several relation types. Counts refer to individual relations and
may overlap; the example does not add them into a misleading unique-publication total.

`disease_treatment_prevention` defaults to `Henoch Schonlein purpura`, which resolves
to IgA vasculitis. You can pass a different disease name and optionally a PubTator
entity ID to disambiguate it. The example prints up to three relations per category,
two supporting papers per relation, and five papers per broader keyword search.
Treatment discovery filters to chemicals; prevention discovery leaves the related
entity type unrestricted. Source/target order is preserved from the relation response.
PubTator's `prevent` labels can describe disease–variant associations, so the example
also performs a separate prevention keyword search rather than treating those labels
as clinical prevention measures.
The keyword searches can include prevention of complications or recurrence; inspect
the papers to determine what was studied. An empty relation list is not evidence
that no treatment or prevention literature exists.

Compose entity and relation queries without string interpolation:

```rust
use pubtator3::{EntityId, RelationEntityType, RelationFilter, RelationType, SearchQuery};

let drug: EntityId = "@CHEMICAL_Doxorubicin".parse()?;
let disease: EntityId = "@DISEASE_Neoplasms".parse()?;
let co_mentions = SearchQuery::entity(&drug).and(SearchQuery::entity(&disease));
let treatment = SearchQuery::relation(RelationFilter::Type(RelationType::Treat), &drug, &disease);
let any_disease = SearchQuery::relation_to_type(RelationFilter::Any, &drug, RelationEntityType::Disease);
```

For pagination, call `SearchResponse::next_page()` until it returns `None` and pass the
returned `Page` to `Client::search`. For large PMID lists, `annotations_batched` performs
sequential requests in chunks of 100. For checkpointing, iterate your own chunks and save
each successful response. Empty ID lists are rejected. Results reflect the documents
available upstream; missing articles are not synthesized or guaranteed to preserve input order.

## Disease synonyms

After choosing the intended autocomplete candidate, pass the `Entity` directly to
`Client::synonyms(&entity)`. It checks the database and descriptor ID before calling
NLM's [MeSH descriptor-details API](https://id.nlm.nih.gov/mesh/swagger/ui).
Genes and entities without a MeSH descriptor ID return a local error. Supplementary
concept IDs (`C…`) are not descriptor IDs (`D…`) and are not supported by this method.

If you already know the MeSH ID, skip autocomplete:

```rust
use pubtator3::{Client, MeshDescriptorId};

let id: MeshDescriptorId = "D000690".parse()?;
let terms = Client::new()?.mesh_synonyms(&id).await?;
if let Some(preferred) = terms.preferred_term() {
    println!("Preferred name: {}", preferred.label);
}
for synonym in terms.synonyms() {
    println!("{}: {}", synonym.id, synonym.label);
}
```

`MeshSynonyms::terms` preserves all returned terms, including the preferred name;
`synonyms()` yields terms explicitly marked nonpreferred. Missing preferred flags
remain `None`, and those terms are not classified as preferred or as synonyms.
These are MeSH entry terms: some describe narrower concepts rather than exact synonyms.
The example rejects ambiguous autocomplete results unless you pass the intended
PubTator entity ID as its second argument.

## Method parameters

Only `autocomplete` and `relations` take request structs. They have optional filters,
so named builder calls are clearer than several positional `Option` arguments:

```rust
let request = AutocompleteRequest::new("cancer")
    .concept(Concept::Disease)
    .limit(NonZeroU32::new(5).unwrap());
let entities = client.autocomplete(&request).await?;
```

Rust has no named or default function arguments. Request builders let callers omit
filters and add new builder options without changing existing calls. They do require
an extra type and construction step; with one or two required arguments, direct
parameters are simpler. `search`, the annotation/export methods, `synonyms(&entity)`,
and `mesh_synonyms(&descriptor)` therefore take their parameters directly.

## Types

| Type | Meaning |
| --- | --- |
| `Pmid` | Positive PubMed ID; accepts numeric or string JSON, serializes as a number |
| `Pmcid` | Full-text identifier with a `PMC` prefix |
| `EntityId` | Validated `@NAMESPACE_name` search accession, including database-qualified forms |
| `MeshDescriptorId`, `MeshTermId` | Validated MeSH descriptor/term IDs, with URI conversion |
| `MeshSynonyms`, `MeshTerm` | MeSH names/entry terms with typed IDs and preferred flags |
| `DocumentId` | BioC local document identifier; separate from a PMID |
| `AnnotationId`, `RelationId` | Document-scoped BioC IDs |
| `Page` | Positive, one-based search page |
| `Concept`, `RelationEntityType`, `RelationType` | Endpoint-specific request enums |
| `TextScope`, `ExportFormat` | Abstract/full text and export format selections |
| `Document`, `Passage`, `Annotation`, `BioCRelation` | Structured BioC responses |

Identifier validation also runs during Serde deserialization. IDs support `Display`,
`FromStr`, equality, ordering, and hashing. String IDs also implement `AsRef<str>`.
Database identifiers in annotation metadata are distinct from PubTator search accessions.
Metadata preserves unknown fields and mixed JSON values (numbers, booleans, nested objects).
BioC relation labels differ from relation-query labels, so `RelationInfo` retains the original
label. Entity categories returned by the service remain strings to accommodate new labels.

`parse_documents` supports the live `{"PubTator3": [...]}` envelope, standard BioC
collections, document arrays, single documents, and streams of JSON documents/collections.
BioC offsets are character offsets, not Rust string byte indices. `RelationNode::refid` is
opaque because PubTator can reference display entries rather than annotation IDs.

## Client behavior

Reuse a client. Clones share the HTTP connection pool and a 350 ms interval between request
starts. Independent clients have independent pacing. `Client::builder()` configures the
API root, HTTP client, per-request timeout (default 60 seconds), and pacing interval.
Supplying a custom HTTP client lets you configure proxy settings or a contact User-Agent.
Synonym requests use `https://id.nlm.nih.gov/mesh/`, configurable with `mesh_base_url`;
`base_url` controls PubTator only. Both services share the timeout, connection pool,
and pacing of a client and its clones.

Exports use documented GET endpoints with a conservative client limit of 100 IDs per call.
Full text uses `full=true` for PMIDs and `publications/pmc_export/…` for PMCIDs.
PubTator text format cannot export full text and is rejected locally for that scope.
XML and PubTator outputs are returned as strings; only BioC JSON is parsed into models.

Transport, JSON, validation, and HTTP failures have distinct error variants. HTTP errors
retain the status, up to 2,048 characters of response body, and the `Retry-After` header.
Requests are not automatically retried; callers can implement their own retry/checkpoint policy.
Any upstream highlighting markup should be escaped or sanitized before rendering in HTML.

## Validation

```sh
cargo test -p pubtator3
cargo clippy -p pubtator3 --all-targets -- -D warnings
```

Offline tests exercise captured official responses, ID validation, query encoding,
pagination, endpoint paths, full-text parameters, pacing, batching, and error handling.
Fixtures were fetched from NCBI on 2026-10-03; the search fixture keeps two results,
and the annotation fixture keeps one complete document. The full-text fixture retains
the title passage, one chemical annotation with parentheses in its accession, and one relation.
The disease fixture captures autocomplete for `Lou Gehrig disease`, and the MeSH fixture
captures `lookup/details?descriptor=D000690&includes=terms`.
