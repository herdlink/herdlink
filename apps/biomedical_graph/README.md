# biomedical_graph

Neo4j storage shared by the existing `pubtator3` and `pubtator3-hpo` clients.
`CachedPubTator` and `CachedHpo` preserve their typed responses, persist fetched
objects and edges, and cache successful queries across process restarts.
Neo4j stores article titles/abstracts and structured PubTator annotations.
Raw API responses and exports are cached on disk; full article text stays out of
the graph. Passage and sentence nodes are not created.
All domain objects have the common `GraphNode` label and a globally unique `uid`.
The graph accepts new node labels and edge types through `GraphBatch`; adding a
new source does not require extending a closed enum or replacing the database.

## Run

The local instance is named `herdlink-neo4j`, with persistent volume
`herdlink-neo4j-data`. Browser: <http://localhost:7474>; Bolt: `127.0.0.1:7687`.
The provided development configuration disables authentication and binds both
ports to localhost. For deployments, configure credentials and TLS using
`neo4rs::ConfigBuilder` and your container configuration.

```sh
# Start on a fresh machine; Docker Compose or Podman Compose can use this file.
docker compose -f apps/biomedical_graph/compose.yaml up -d

# Alternative using Podman directly, matching the supplied Compose configuration:
podman run -d --name herdlink-neo4j \
  -p 127.0.0.1:7474:7474 -p 127.0.0.1:7687:7687 \
  -e NEO4J_AUTH=none -e NEO4J_dbms_usage__report_enabled=false \
  -v herdlink-neo4j-data:/data docker.io/library/neo4j:5.26-community

# Four bounded queries: two autocomplete hits, two gene relations, one abstract,
# and one HPO disease mapping. Output is limited to five phenotype labels.
cargo run -p biomedical_graph --example sample

# Use a complete existing HPO/Mondo snapshot for the queried disease:
cargo run -p biomedical_graph --example sample -- /path/to/phenotype-data
```

The default example uses an **incomplete official subset**: four Huntington HPO
annotation rows, fourteen ontology terms and one Mondo node. It demonstrates the
schema, not the full phenotype profile or a similarity corpus. The explicit path
accepts the existing client's `hp.obo`, `phenotype.hpoa`, and `mondo.json` files.
Environment variables in [.env.example](.env.example) configure the example;
they must be exported in the shell (the crate does not automatically load `.env`).
`Store::connect` installs the idempotent constraints and index in
[schema.cypher](schema.cypher). The default Community database is `neo4j`.
Raw responses use `.graph-query-cache/` by default. Set `HERDLINK_QUERY_CACHE_DIR`
or use `Store::connect_with_cache_dir` to choose a persistent cache directory.
All application instances sharing Neo4j should share that directory to reuse
cached responses. Missing local files cause a fresh upstream fetch.

## Client use

```rust,no_run
use biomedical_graph::{CachePolicy, CachedHpo, CachedPubTator, Store};
use pubtator3::{AutocompleteRequest, Concept};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = neo4rs::ConfigBuilder::default()
    .uri("127.0.0.1:7687").user("neo4j").password("").db("neo4j").build()?;
let store = Store::connect(config).await?;
let upstream = pubtator3::Client::new()?;
let pubtator = CachedPubTator::new(upstream.clone(), store.clone(), CachePolicy::default());

let request = AutocompleteRequest::new("Huntington disease").concept(Concept::Disease);
let candidates = pubtator.autocomplete(&request).await?;
// Repeating this request returns the persisted response, without an HTTP call.
let cached_candidates = pubtator.autocomplete(&request).await?;

let dataset = tokio::task::spawn_blocking(|| pubtator3_hpo::Dataset::from_dir("./phenotype-data"))
    .await??;
let hpo = CachedHpo::new(pubtator3_hpo::Client::new(upstream, dataset), store, CachePolicy::default());
let profile = hpo.disease_phenotypes("Huntington disease", None).await?;
# Ok(())
# }
```

`CachedPubTator` wraps autocomplete, relation discovery, paginated search,
abstract/full-text annotations, batching, PMC annotations, MeSH synonyms, and raw
exports. BioC JSON exports also project their structured annotations. XML and
PubTator text exports have `RawExport` metadata nodes; their bodies are on disk.
Use `annotations` for structured nodes. Cached methods return the complete original
typed response, including full text when requested, without storing it in Neo4j.
`CachedHpo` wraps disease resolution, phenotype profiles, similarity reports,
reverse PubTator mapping and supporting-paper searches. Its additional `profile`
method caches a local lookup by typed disease ID without making HTTP requests.
HPO client methods run the existing mapper/ranker; their successful returned
objects are projected. Intermediate autocomplete candidates discarded inside
that client are not exposed; call `CachedPubTator::autocomplete` when you want
the complete candidate response stored.

The `hpo` dependency itself is an in-memory ontology library rather than a network
client. This wrapper targets the repository's `pubtator3-hpo::Client`, which loads
that ontology and HPO annotations and maps them through Mondo.

## Function tools and runnable example

`biomedical_graph::tools::GraphTools` declares strict Responses API function
schemas and dispatches JSON arguments through the cached clients. It registers
eleven PubTator tools by default; `.with_hpo(cached_hpo)` adds six HPO tools.
Successful results retain the complete original response. `execute_with_graph` additionally returns the exact structured projection for supported website tools, including cache hits, without scanning Neo4j. The backend uses these projections to update only the current conversation’s graph. `execute(name, json)`
returns a JSON value or error; `call(&FunctionCall)` returns an
`openai::InputItem` with the original call ID and a result or `{"error": "…"}`.

| Tool name | Cached method |
| --- | --- |
| `pubtator_autocomplete` | `CachedPubTator::autocomplete` |
| `pubtator_relations` | `CachedPubTator::relations` |
| `pubtator_search` | `CachedPubTator::search` |
| `pubtator_relation_papers` | Oriented relation search, persisted with `RelationEvidence` / `SUPPORTED_BY` |
| `pubtator_annotations` | `CachedPubTator::annotations` |
| `pubtator_annotations_batched` | `CachedPubTator::annotations_batched` |
| `pubtator_pmc_annotations` | `CachedPubTator::pmc_annotations` |
| `pubtator_mesh_synonyms` | `CachedPubTator::mesh_synonyms` |
| `pubtator_synonyms` | `CachedPubTator::synonyms` |
| `pubtator_export` | `CachedPubTator::export` |
| `pubtator_pmc_export` | `CachedPubTator::pmc_export` |
| `hpo_resolve_disease` | `CachedHpo::resolve_disease` |
| `hpo_disease_phenotypes` | `CachedHpo::disease_phenotypes` |
| `hpo_profile` | `CachedHpo::profile` |
| `hpo_similar_diseases` | `CachedHpo::similar_diseases` |
| `hpo_pubtator_entities` | `CachedHpo::pubtator_entities` |
| `hpo_supporting_papers` | `CachedHpo::supporting_papers` |

```sh
# Print every name, description and JSON argument schema; no database needed.
cargo run -p biomedical_graph --example tool_calls -- --list

# With Neo4j running: exercise all 17 tools and compare results from fresh clients.
# Cache misses call PubTator/MeSH; no OpenAI credentials are needed.
cargo run -p biomedical_graph --example tool_calls -- --smoke

# Dispatch a specific tool directly using JSON arguments.
cargo run -p biomedical_graph --example tool_calls -- --call pubtator_autocomplete \
  '{"query":"Huntington disease","concept":"disease","limit":2}'

# Optional model loop; set OPENAI_MODEL plus OPENAI_API_KEY or OPENAI_AUTH_FILE.
cargo run -p biomedical_graph --example tool_calls -- --prompt \
  'Find Huntington disease, two associated genes, and its HPO phenotype profile.'

# Recorded responses + real Neo4j: all tools, batching, exports, offline cache replay.
cargo test -p biomedical_graph --test tools -- --ignored
```

The example reads the existing `NEO4J_*` and `HERDLINK_QUERY_CACHE_DIR` variables.
`HPO_DATA_DIR` selects an existing full HPO/Mondo snapshot; otherwise it uses the
bundled incomplete fixture subset, which cannot demonstrate a similarity corpus.
The smoke test reports `SKIP hpo_similar_diseases` for a corpus with no informative
terms; set `HPO_DATA_DIR` to a full snapshot to verify a successful similarity call.
`PUBTATOR_BASE_URL` and `MESH_BASE_URL` optionally select proxy/test API roots.
The model mode makes real OpenAI requests and consumes account usage.
Full-text/raw results can be large. Smoke tests report byte counts, while direct
calls and the model loop receive complete results. The smoke test stops at the
first failure and reports the tool name; fetched graph/cache data remains intact.
`pubtator_synonyms` accepts only `_id`, `name`, `biotype`, `db`, `db_id` from an
autocomplete entity. `hpo_pubtator_entities` accepts a complete similarity match.
For model calls every declared property is required; optional filters use `null`.
Constructors, cache eviction, graph writes and arbitrary Cypher are not tools.

```rust,no_run
use biomedical_graph::tools::GraphTools;
use openai::{ResponseRequest, FunctionCall};
# async fn example(pubtator: biomedical_graph::CachedPubTator,
#     hpo: biomedical_graph::CachedHpo, call: &FunctionCall) -> biomedical_graph::Result<()> {
let tools = GraphTools::new(pubtator).with_hpo(hpo);
let mut request = ResponseRequest::new("your-model", "Find papers about HTT.");
request.tools = tools.definitions();
// Inside a response.tool_calls() loop:
let output = tools.call(call).await;
// Collect outputs and pass them to request.continue_from(&response, outputs).
# let _ = output;
# Ok(())
# }
```

## Node types and data

| Labels | Identity and retained data |
| --- | --- |
| `Entity` + `Gene`, `DiseaseEntity`, `Chemical`, `Variant`, `Species`, `CellLine`, `OtherEntity` | PubTator `@…` accession; names, biotype, database identifiers, descriptions, matching text and extra metadata. `DiseaseEntity` is a PubTator representation; `Disease` is the mapped curated concept. Unknown categories keep their raw values. |
| `Identifier`, optionally `MeshDescriptor` | Namespaced MeSH, DOI, PMC, other database or citation identifier. Database identifiers remain distinct from PubTator search accessions. Multiple/unknown annotation identifiers stay opaque. |
| `MeshTerm` | MeSH term ID, entry label and optional preferred flag; entry terms are not automatically exact synonyms. |
| `Publication` | PMID; title, abstract, journal, author strings, date, DOI, PMCID and citations when available. Missing metadata from a partial response does not erase known text. |
| `Document` | Namespace + text scope + content hash. Local BioC ID, title/abstract from explicitly classified sections, PMID/PMCID links. Abstract/full-text annotations remain separate document versions; body text and full response JSON are not graph properties. |
| `Mention`, `MentionLocation` | PubTator-supplied annotation ID, surface text, normalized entity fields and offset/length spans. Directly linked to the document. Offsets are characters, not UTF-8 bytes; nothing is inferred from text. |
| `ExtractedRelation` | Owner-scoped BioC relation ID; original label, original score representation, container identifier and structured participants. |
| `BioCReference` | Opaque BioC `refid` and role. An upstream display reference is not assumed to identify a mention. |
| `RelationSummary` | Content-addressed PubTator relation-discovery record with ordered source/target, query relation type and supporting-publication count. Counts are not scores or unique sums across overlapping relations. |
| `Disease` | Canonical `MONDO:…` disease or independent OMIM/ORPHA/DECIPHER disease when no unique exact Mondo grouping exists. |
| `DiseaseRecord` | Original HPO disease annotation record ID. OMIM/ORPHA records are grouped only according to the existing exact mapper. |
| `HpoDiseaseProfile` | Disease + dataset fingerprint; complete typed profile, positive/absent/conflicting features, constituent records and unscored count. |
| `HpoTerm`, optionally `HpoAlias` | `HP:…` ID, name, parents, obsolete status, replacement IDs, alternative IDs and original OBO tags, including definitions, scoped synonyms, cross-references and future tags. All ontology branches are retained. |
| `HpoAnnotation` | Snapshot + profile-local row identity; disease record, phenotype, exclusion, reference, evidence, onset, frequency, sex, modifier, aspect and biocuration. Duplicate normalized annotation rows remain separate row objects. |
| `DiseaseMapping` | PubTator entity → curated disease mapping with `ExactMesh`, `ExactName`, `AnnotationId` or `Override`, matched value and mapping configuration. Lexical/override matches remain distinguishable from exact identifier evidence. |
| `DatasetSnapshot` | SHA-256 of all three actual input files and the HPO version. Manual snapshots also have a fingerprint. |
| `SimilarityResult` | Source/target disease, simGIC score, shared/source-only/conflicting phenotypes, algorithm, options, dataset fingerprint and corpus size. This is a computed result, not a curated biological relationship. |
| `FetchResult`, `QueryCache` | Operation, parameters, upstream/configuration identity, SHA-256 reference to the disk response file, payload size, cache TTL and server-clock timestamps. |
| `RawExport` | Format, scope, content hash and byte count. The XML, text or BioC JSON body is cached on disk. |

Nested maps and lists are serialized as JSON strings because Neo4j properties
cannot contain arbitrary nested objects. Scalars remain directly queryable;
out-of-range unsigned integers are strings to avoid precision loss. Complete
responses remain in content-addressed `<payload_hash>.json` files, including
ranking, highlighting, pagination, body text and unknown fields. Cache reads
verify the SHA-256 checksum. HPO OBO metadata on a shared term
reflects its latest upsert; snapshot-qualified hierarchy edges and profile payloads
preserve their source context. This is not a fully temporal ontology archive.

## Relationship types

| Edge | Direction and meaning |
| --- | --- |
| `IDENTIFIED_BY` | Entity/publication/document → namespaced identifier. |
| `HAS_ENTRY_TERM` | MeSH descriptor → entry term, with the upstream preferred flag. |
| `HAS_DOCUMENT` | Publication → document containing a source-scoped set of structured annotations. |
| `HAS_MENTION`, `HAS_LOCATION`, `DENOTES` | Document → supplied mention → span; mention → normalized entity or opaque database identifier. |
| `HAS_RELATION` | Document → extracted relation. A container identifier property preserves the original document/passage/sentence scope without creating those content nodes. |
| `HAS_PARTICIPANT` | Extracted relation → entity, with the original `role1`/`role2` property. |
| `HAS_REFERENCE` | Extracted relation → opaque BioC reference. |
| `SOURCE`, `TARGET` | Relation summary or similarity result → the corresponding endpoint, preserving upstream orientation. |
| `PUBTATOR_RELATION` | Source entity → target entity; `relation_type`, `publications`, `summary_uid` retain the summary evidence. A distinct edge per evidence record preserves refresh history. |
| `PROFILE_OF`, `HAS_RECORD`, `RECORD_OF` | Profile → canonical disease; profile → source record; source record → grouped disease, qualified by snapshot. |
| `EXACT_MESH_MAPPING` | Canonical disease → MeSH ID, only from exact Mondo mapping evidence, qualified by snapshot. |
| `HAS_ANNOTATION`, `ANNOTATES_RECORD`, `PHENOTYPE` | Profile → HPO annotation → original disease record / referenced HPO term. Includes unscored and non-phenotype aspects. |
| `HAS_PHENOTYPE`, `EXCLUDES_PHENOTYPE`, `CONFLICTING_PHENOTYPE` | Profile → scored positive, explicitly absent or contradictory term. These are derived conveniences; evidence stays on annotation nodes. |
| `ONSET`, `FREQUENCY`, `MODIFIER` | HPO annotation → HPO term when the corresponding field contains an HP ID. Percentages/fractions and original multi-value strings remain properties. |
| `SUPPORTED_BY` | HPO annotation → PubMed publication or other citation identifier. |
| `IS_A` | HPO child → **direct** parent, with `snapshot_uid`. Ancestor closure is traversed, not stored as false direct-parent edges. |
| `REPLACED_BY`, `NORMALIZES_TO` | Obsolete term → stated replacement; alternative/uniquely resolved obsolete ID → active ID. Multiple stated replacements do not become a unique normalization. |
| `IN_SNAPSHOT` | HPO profile/term → dataset snapshot. |
| `MAPPED_VIA`, `MAPS_TO` | PubTator entity → mapping evidence → canonical disease. |
| `CACHED_RESULT`, `HAS_OBJECT` | Cache entry → immutable result → each projected object. Historical response provenance survives cache expiry/eviction. |

Supported relation-query labels are `treat`, `cause`, `cotreat`, `convert`,
`compare`, `interact`, `associate`, `positive_correlate`, `negative_correlate`,
`prevent`, `inhibit`, `stimulate`, and `drug_interact`. BioC extraction labels
such as `Positive_Correlation` retain their original spelling. Relations use a
property on a stable edge type instead of converting upstream strings into
Cypher syntax. Source/target order is preserved even for symmetric relations.

Co-mention searches and similarity do not produce causal/treatment edges.
Annotation scores, search ranking scores and relation-summary publication counts
retain their different meanings. Absent annotations stay separate from positive
phenotypes. A missing phenotype annotation does not create an absence edge.
Plain Mondo cross-references and related synonyms do not create equivalence edges.

## Cache guarantees and extension points

- Default TTL: 24 hours, configurable with `CachePolicy`. Zero TTL forces refresh.
- Cache keys include operation and every request parameter: query/page, filters,
  limits, IDs in their given order, export format and text scope. PubTator roots
  isolate proxy/test servers; HPO keys also include exact dataset content, name
  fallback and reviewed overrides. A schema/projection version prefixes all keys.
- A successful miss installs an immutable response file, then projects objects
  and publishes its cache reference in **one Neo4j transaction**. Graph or HTTP
  errors never publish a partial successful entry. Filesystem and Neo4j writes
  are separate: failed graph writes or crashes can leave unreferenced disk files.
  Missing/empty upstream results are cached as returned.
- Clones serialize their cache work and coalesce identical misses. Independently
  created clients/processes can duplicate HTTP fetches; stable identities and
  transactional `MERGE` keep nodes idempotent. Cross-process fetch locking and
  automatic HTTP retries are not implemented.
- Expiry uses the Neo4j server clock. Refresh replaces `CACHED_RESULT` atomically;
  immutable fetch/document/evidence metadata remains for provenance. Cache
  eviction/pruning does not delete biological objects or disk files. Historical
  metadata and disk responses require a separate retention policy.
- `cache_key(operation, request_tuple)` plus `Store::invalidate` evicts a specific
  query. Tuple layouts are visible in `clients.rs`. `Store::prune_cache` removes
  expired entries. `Store::graph` permits custom Cypher/migrations.
- Requested HPO profiles include only their terms, metadata terms and ancestor
  closure. `Store::import_hpo(&dataset)` optionally imports the entire loaded
  ontology and every disease profile in bounded, independently resumable
  transactions. It makes no HTTP calls. The full import is not one transaction.

```rust,no_run
use biomedical_graph::{GraphBatch, Properties};
use serde_json::json;
# async fn extend(store: biomedical_graph::Store) -> biomedical_graph::Result<()> {
let mut batch = GraphBatch::default();
let trial = batch.node("trial:NCT00000001", "ClinicalTrial",
    Properties::from([("phase".into(), json!(2))]));
let disease = batch.node("disease:MONDO:0007739", "Disease", Properties::new());
batch.edge(&trial, "STUDIES", &disease, "registry-v1", Properties::new());
store.upsert(&batch).await?;
# Ok(())
# }
```

Labels and edge types must match `[A-Za-z][A-Za-z0-9_]*`; all data values are
parameterized. `uid` is reserved. Include both endpoints in each batch; existing
endpoints can be represented by nodes with empty properties. Node IDs include
their namespaces; future sources should choose a stable identity convention.

This models the datasets currently loaded by the repository. HPO gene–phenotype
or gene–disease files are **not** part of the existing client's snapshot and are
not inferred from PubTator associations. Authors currently arrive as strings, so
they remain publication properties rather than guessed person identities.
Whole-Mondo ontology hierarchy and fuzzy disease equivalence are also outside
the current mapper's output; add explicit importers when those sources are needed.

## Inspect the graph

```cypher
// A PubTator disease's curated phenotype evidence, including absence and references.
MATCH (e:Entity {accession: '@DISEASE_Huntington_Disease'})
      -[:MAPPED_VIA]->(mapping:DiseaseMapping)-[:MAPS_TO]->(d:Disease)
MATCH (p:HpoDiseaseProfile)-[:PROFILE_OF]->(d)
MATCH (p)-[:HAS_ANNOTATION]->(a:HpoAnnotation)-[:PHENOTYPE]->(t:HpoTerm)
RETURN d.name, mapping.method, t.id, t.name, a.excluded, a.evidence, a.reference;

// Current cached relation-discovery result, avoiding historical refresh counts.
MATCH (c:QueryCache)-[:CACHED_RESULT]->(f:FetchResult)-[:HAS_OBJECT]->(r:RelationSummary)
WHERE c.expires_at > timestamp()
MATCH (r)-[:SOURCE]->(s:Entity), (r)-[:TARGET]->(t:Entity)
RETURN s.accession, r.type, t.accession, r.publications;

// Direct HPO parent hierarchy from one explicitly selected snapshot.
MATCH (child:HpoTerm {id:'HP:0002072'})-[r:IS_A]->(parent:HpoTerm)
RETURN child.name, parent.id, parent.name, r.snapshot_uid;

// Mentions and character spans in one publication.
MATCH (:Publication {pmid:19894120})-[:HAS_DOCUMENT]->(d:Document)
MATCH (d)-[:HAS_MENTION]->(m:Mention)
MATCH (m)-[:HAS_LOCATION]->(loc:MentionLocation)
RETURN d.scope, m.text, loc.offset, loc.length;
```

## Samples and verification

On 2026-10-04, bounded live requests returned Huntington autocomplete (two
candidates), two gene associations (`HTT`: 3,616 publications; `BDNF`: 189),
PMID `19894120` (one abstract, 26 mentions), and HPO Chorea `HP:0002072`.
Fixtures retain complete bounded responses. The HPO/Mondo fixture subsets were
extracted from the already available official `v2026-09-01` files; no full dataset
was downloaded again. See [fixture provenance](tests/fixtures/README.md).

```sh
cargo test -p biomedical_graph -p pubtator3 -p pubtator3-hpo
cargo test -p biomedical_graph --test neo4j -- --ignored
cargo clippy -p biomedical_graph -p pubtator3 -p pubtator3-hpo --all-targets -- -D warnings
```

Projection tests cover captured data, scoped IDs, directed relation counts,
opaque references, title/abstract allowlisting, excluded body/sentence text,
direct parents, HPO normalization, negative annotations,
mapping configuration and source fingerprints. The optional integration test
uses real Neo4j plus a localhost HTTP server to verify persistent hits, concurrent
miss coalescing, TTL, invalidation, offline hits, transaction rollback and adding
a future node/edge type. It also verifies complete full-text cache round trips
without body text in any graph properties. It deletes its new graph nodes after
succeeding and uses a temporary response directory.

Source formats: [PubTator3 API](https://www.ncbi.nlm.nih.gov/research/pubtator3/api),
[HPO annotations](https://obophenotype.github.io/human-phenotype-ontology/annotations/phenotype_hpoa/),
[Mondo mapping guidance](https://mondo.monarchinitiative.org/pages/faq/).

## Website research examples

`just graph-seed` runs the bounded, repeatable `seed_use_cases` example for
Huntington disease, Parkinson disease and ALS. For each exact PubTator match it
persists up to two gene associations, one page of oriented relation papers and
one annotated abstract. When a full HPO snapshot is available it also persists
two phenotype matches and their curated annotation citations. A missing disease
mapping is reported; it is never replaced by a guessed profile. Data comes from
the real cached clients; no synthetic example data is inserted. Existing nodes
and query caches are reused. `just graph-data` downloads official release files
into an empty `phenotype-data` directory if a full snapshot is not already present.

For 50 additional disease use cases, run from the repository root with the same
exported `NEO4J_*`, `HPO_DATA_DIR` and cache settings as the backend:

```sh
just graph-seed-more

# Without just, optionally using a release build on a server:
cargo run --release -p biomedical_graph --example seed_use_cases -- \
  --diseases-file apps/biomedical_graph/examples/more-diseases.txt

# Print the exact list without connecting to Neo4j or calling external APIs:
cargo run -p biomedical_graph --example seed_use_cases -- \
  --diseases-file apps/biomedical_graph/examples/more-diseases.txt --list

# Use a custom file (one exact PubTator name per line):
just graph-seed-more /path/to/diseases.txt
```

The [50-disease list](examples/more-diseases.txt) uses canonical PubTator names,
including Hepatolenticular Degeneration for Wilson disease and Glycogen Storage
Disease Type II for Pompe disease. Blank lines and `#` comments are ignored;
duplicate names are processed once. Files accept 1–200 distinct names. Each use
case keeps the same bounded gene, literature and optional phenotype enrichment;
annotations and phenotype matches may introduce additional disease nodes.

The seed prints progress and a final seeded/skipped/failed count. Individual
failures do not stop later diseases from being attempted, and unresolved or
failed imports produce a nonzero exit status. Rerun safely to reuse existing
nodes and query caches. HPO mapping failures are reported separately and do not
prevent PubTator data from being seeded. No OpenAI credentials are needed.
