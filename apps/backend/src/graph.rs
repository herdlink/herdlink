//! Read-only, bounded projections of stored Neo4j data for the graph viewer.
use crate::{
    AppState,
    auth::AuthUser,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use neo4rs::{Graph, query};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};
use tokio::sync::OnceCell;

const DISPLAY_LABELS: &[&str] = &[
    "Entity",
    "Disease",
    "HpoTerm",
    "Publication",
    "DatasetSnapshot",
    "ClinicalTrial",
];
const RAW_LIMIT: usize = 2000;
const EDGE_LIMIT: usize = 5000;

#[derive(Clone)]
pub struct GraphSettings {
    pub uri: String,
    pub user: String,
    pub password: String,
    pub database: String,
}
impl GraphSettings {
    pub fn from_env() -> Self {
        Self {
            uri: std::env::var("NEO4J_URI").unwrap_or_else(|_| "127.0.0.1:7687".into()),
            user: std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into()),
            password: std::env::var("NEO4J_PASSWORD").unwrap_or_default(),
            database: std::env::var("NEO4J_DATABASE").unwrap_or_else(|_| "neo4j".into()),
        }
    }
}
pub struct GraphReader {
    settings: GraphSettings,
    connection: OnceCell<Graph>,
}
impl GraphReader {
    pub fn new(settings: GraphSettings) -> Self {
        Self {
            settings,
            connection: OnceCell::new(),
        }
    }
    pub(crate) async fn community_name(&self, slug: &str) -> Option<String> {
        let (namespace, code) = slug.split_once('-')?;
        if !matches!(namespace, "mondo" | "omim" | "orpha") || code.contains('-') {
            return None;
        }
        let read = async {
            let graph = self.connection().await.ok()?;
            let mut rows = graph
                .execute(
                    query("MATCH (n:GraphNode:Disease {id: $id}) RETURN n.name AS name LIMIT 1")
                        .param("id", format!("{namespace}:{code}").to_ascii_uppercase()),
                )
                .await
                .ok()?;
            let row = rows.next().await.ok()??;
            let name: String = row.get("name").ok()?;
            let name = name.trim();
            (!name.is_empty() && !name.contains('\0')).then(|| name.chars().take(100).collect())
        };
        tokio::time::timeout(Duration::from_secs(2), read)
            .await
            .ok()
            .flatten()
    }
    async fn connection(&self) -> Result<&Graph> {
        self.connection
            .get_or_try_init(|| async {
                let config = neo4rs::ConfigBuilder::default()
                    .uri(&self.settings.uri)
                    .user(&self.settings.user)
                    .password(&self.settings.password)
                    .db(self.settings.database.as_str())
                    .max_connections(4)
                    .build()
                    .map_err(AppError::internal)?;
                Graph::connect(config).await.map_err(graph_error)
            })
            .await
    }
}
fn graph_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(%error, "graph read failed");
    AppError(
        StatusCode::SERVICE_UNAVAILABLE,
        "The graph database is unavailable. Check Neo4j and try again.",
    )
}

#[derive(Default, Deserialize)]
pub struct SnapshotQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub result_uid: String,
    pub limit: Option<usize>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ViewNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub labels: Vec<String>,
    pub description: Option<String>,
    pub links: Vec<Link>,
    pub community_url: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ViewEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub kind: String,
    pub evidence_id: String,
    pub evidence_kind: String,
    pub reported_papers: Option<u64>,
    pub context: Value,
}
#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub nodes: Vec<ViewNode>,
    pub edges: Vec<ViewEdge>,
    pub query: String,
    pub result_uid: String,
    pub truncated: bool,
    pub generated_at: String,
}
#[derive(Clone)]
struct RawNode {
    id: String,
    labels: Vec<String>,
    props: Value,
}
#[derive(Clone)]
struct RawEdge {
    id: String,
    source: String,
    target: String,
    kind: String,
    props: Value,
}
fn text(props: &Value, key: &str) -> Option<String> {
    match props.get(key)? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn has(node: &RawNode, label: &str) -> bool {
    node.labels.iter().any(|l| l == label)
}
fn display(node: &RawNode) -> bool {
    DISPLAY_LABELS.iter().any(|label| has(node, label))
}
fn kind(node: &RawNode) -> &'static str {
    if has(node, "Disease") || has(node, "DiseaseEntity") {
        "disease"
    } else if has(node, "Gene") {
        "gene"
    } else if has(node, "Chemical") {
        "chemical"
    } else if has(node, "HpoTerm") {
        "phenotype"
    } else if has(node, "Publication") {
        "publication"
    } else if has(node, "Variant") {
        "variant"
    } else if has(node, "Species") {
        "species"
    } else if has(node, "CellLine") {
        "cell_line"
    } else if has(node, "ClinicalTrial") {
        "clinical_trial"
    } else if has(node, "DatasetSnapshot") {
        "dataset"
    } else {
        "other"
    }
}
fn label(node: &RawNode) -> String {
    if has(node, "DatasetSnapshot") {
        return format!(
            "HPO dataset {}",
            text(&node.props, "fingerprint")
                .unwrap_or_default()
                .chars()
                .take(8)
                .collect::<String>()
        );
    }
    for key in ["name", "title", "label", "accession", "id", "pmid"] {
        if let Some(value) = text(&node.props, key) {
            return if key == "pmid" {
                format!("PMID {value}")
            } else if key == "accession" {
                value
                    .trim_start_matches('@')
                    .split_once('_')
                    .map(|(_, name)| name.replace('_', " "))
                    .unwrap_or(value)
            } else {
                value
            };
        }
    }
    node.id.clone()
}
fn url(base: &str, name: &str, value: &str) -> String {
    let mut link = url::Url::parse(base).expect("static URL");
    link.query_pairs_mut().append_pair(name, value);
    link.to_string()
}
fn source_links(node: &RawNode) -> Vec<Link> {
    let mut links = Vec::new();
    let mut add = |label: &str, url: String| {
        links.push(Link {
            label: label.into(),
            url,
        })
    };
    if let Some(pmid) = text(&node.props, "pmid") {
        add(
            "PubMed article",
            format!(
                "https://pubmed.ncbi.nlm.nih.gov/{}/",
                url::form_urlencoded::byte_serialize(pmid.as_bytes()).collect::<String>()
            ),
        );
    }
    if let Some(doi) = text(&node.props, "doi") {
        add(
            "DOI",
            format!(
                "https://doi.org/{}",
                url::form_urlencoded::byte_serialize(doi.as_bytes()).collect::<String>()
            ),
        );
    }
    if let Some(pmcid) = text(&node.props, "pmcid") {
        add(
            "Full text in PMC",
            format!("https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/"),
        );
    }
    if let Some(id) = text(&node.props, "id") {
        if id.starts_with("HP:") {
            add(
                "Human Phenotype Ontology",
                format!("https://hpo.jax.org/browse/term/{id}"),
            );
        } else if id.starts_with("MONDO:") {
            add(
                "Mondo disease ontology",
                url(
                    "https://www.ebi.ac.uk/ols4/ontologies/mondo/terms",
                    "obo_id",
                    &id,
                ),
            );
        } else if let Some(omim) = id.strip_prefix("OMIM:") {
            add("OMIM", format!("https://omim.org/entry/{omim}"));
        }
    }
    if let Some(id) = text(&node.props, "db_id") {
        match text(&node.props, "db").as_deref() {
            Some("ncbi_mesh") => add(
                "MeSH record",
                url("https://meshb.nlm.nih.gov/record/ui", "ui", &id),
            ),
            Some("ncbi_gene") => add(
                "NCBI Gene",
                format!("https://www.ncbi.nlm.nih.gov/gene/{id}"),
            ),
            _ => {}
        }
    }
    if let Some(accession) = text(&node.props, "accession") {
        add(
            "PubTator source",
            url(
                "https://www.ncbi.nlm.nih.gov/research/pubtator3/",
                "query",
                &accession,
            ),
        );
    }
    if kind(node) == "dataset" {
        add(
            "HPO source dataset",
            "https://hpo.jax.org/data/annotations".into(),
        );
    }
    add(
        "Search related literature",
        url("https://pubmed.ncbi.nlm.nih.gov/", "term", &label(node)),
    );
    links
}
fn community_key(node: &RawNode) -> String {
    let identity = text(&node.props, "community_id")
        .or_else(|| text(&node.props, "id"))
        .or_else(|| {
            text(&node.props, "db_id").map(|id| {
                format!(
                    "{}-{id}",
                    text(&node.props, "db").unwrap_or_else(|| "disease".into())
                )
            })
        })
        .unwrap_or_else(|| node.id.clone());
    let normalized = identity
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let slug = normalized
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if !slug.is_empty() && slug.len() <= 80 {
        slug
    } else {
        format!(
            "disease-{:x}",
            node.id
                .bytes()
                .fold(0xcbf29ce484222325u64, |h, b| (h ^ u64::from(b))
                    .wrapping_mul(0x100000001b3))
        )
    }
}

pub async fn snapshot(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(input): Query<SnapshotQuery>,
) -> Result<Json<Snapshot>> {
    if input.q.chars().count() > 200 || input.result_uid.len() > 500 || input.q.contains('\0') {
        return Err(AppError::bad_request("invalid graph query"));
    }
    let limit = input.limit.unwrap_or(180);
    if !(1..=400).contains(&limit) {
        return Err(AppError::bad_request("graph limit must be 1-400"));
    }
    tokio::time::timeout(
        Duration::from_secs(15),
        read_snapshot(&state.graph, input, limit),
    )
    .await
    .map_err(|_| graph_error("read timed out"))?
    .map(Json)
}
async fn read_snapshot(
    reader: &GraphReader,
    input: SnapshotQuery,
    limit: usize,
) -> Result<Snapshot> {
    // An unscoped read must never become a database-wide overview.
    if input.result_uid.trim().is_empty() {
        return read_disease(reader, input).await;
    }
    let graph = reader.connection().await?;
    let mut stream = graph.execute(query("MATCH (n:GraphNode) WHERE any(label IN labels(n) WHERE label IN $labels) AND ($q = '' OR any(value IN [n.uid, n.name, n.title, n.accession, n.id] WHERE toLower(toString(value)) CONTAINS toLower($q))) AND EXISTS { MATCH (:GraphNode:FetchResult {uid: $result})-[:HAS_OBJECT]->(n) } RETURN n.uid AS id, labels(n) AS labels, properties(n) AS props ORDER BY CASE WHEN n:Disease THEN 0 WHEN n:Entity THEN 1 WHEN n:HpoTerm THEN 2 ELSE 3 END, n.uid LIMIT $limit")
        .param("labels", DISPLAY_LABELS.iter().map(|s| s.to_string()).collect::<Vec<_>>()).param("q", input.q.trim()).param("result", input.result_uid.clone()).param("limit", (limit + 1) as i64)).await.map_err(graph_error)?;
    let mut nodes = BTreeMap::new();
    while let Some(row) = stream.next().await.map_err(graph_error)? {
        let node = RawNode {
            id: row.get("id").map_err(graph_error)?,
            labels: row.get("labels").map_err(graph_error)?,
            props: row.get("props").map_err(graph_error)?,
        };
        nodes.insert(node.id.clone(), node);
    }
    let mut truncated = nodes.len() > limit;
    let seeds = nodes.keys().cloned().collect::<BTreeSet<_>>();
    // Bounded breadth-first reads avoid unbounded variable-length path expansion.
    let mut frontier = nodes.keys().cloned().collect::<Vec<_>>();
    for _ in 0..3 {
        if frontier.is_empty() || nodes.len() >= RAW_LIMIT {
            break;
        }
        let remaining = RAW_LIMIT - nodes.len();
        let mut next = Vec::new();
        let mut stream = graph.execute(query("MATCH (seed:GraphNode)-[]-(n:GraphNode) WHERE seed.uid IN $frontier AND NOT n.uid IN $known AND NOT (n:QueryCache OR n:FetchResult OR n:MentionLocation OR n:RawExport OR n:BioCReference) RETURN DISTINCT n.uid AS id, labels(n) AS labels, properties(n) AS props ORDER BY n.uid LIMIT $limit")
            .param("frontier", frontier).param("known", nodes.keys().cloned().collect::<Vec<_>>()).param("limit", remaining as i64)).await.map_err(graph_error)?;
        while let Some(row) = stream.next().await.map_err(graph_error)? {
            let node = RawNode {
                id: row.get("id").map_err(graph_error)?,
                labels: row.get("labels").map_err(graph_error)?,
                props: row.get("props").map_err(graph_error)?,
            };
            next.push(node.id.clone());
            nodes.insert(node.id.clone(), node);
        }
        if next.len() == remaining {
            truncated = true;
        }
        frontier = next;
    }
    let mut edges = Vec::new();
    if !nodes.is_empty() {
        let mut stream = graph.execute(query("MATCH (a:GraphNode)-[r]->(b:GraphNode) WHERE a.uid IN $ids AND b.uid IN $ids RETURN r.uid AS id, a.uid AS source, b.uid AS target, type(r) AS kind, properties(r) AS props ORDER BY r.uid LIMIT $limit")
            .param("ids", nodes.keys().cloned().collect::<Vec<_>>()).param("limit", (EDGE_LIMIT + 1) as i64)).await.map_err(graph_error)?;
        while let Some(row) = stream.next().await.map_err(graph_error)? {
            edges.push(RawEdge {
                id: row.get("id").map_err(graph_error)?,
                source: row.get("source").map_err(graph_error)?,
                target: row.get("target").map_err(graph_error)?,
                kind: row.get("kind").map_err(graph_error)?,
                props: row.get("props").map_err(graph_error)?,
            });
        }
    }
    if edges.len() > EDGE_LIMIT {
        truncated = true;
        edges.truncate(EDGE_LIMIT);
    }
    let (view_nodes, view_edges, clipped) = project(&nodes, &edges, &seeds, limit);
    Ok(Snapshot {
        nodes: view_nodes,
        edges: view_edges,
        query: input.q,
        result_uid: input.result_uid,
        truncated: truncated || clipped,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

async fn read_disease(reader: &GraphReader, input: SnapshotQuery) -> Result<Snapshot> {
    let mut snapshot = Snapshot {
        nodes: Vec::new(),
        edges: Vec::new(),
        query: input.q.clone(),
        result_uid: String::new(),
        truncated: false,
        generated_at: chrono::Utc::now().to_rfc3339(),
    };
    let term = input.q.trim();
    if term.is_empty() {
        return Ok(snapshot);
    }
    let graph = reader.connection().await?;
    // Prefer an exact name/identifier, then a canonical disease. Return one disease,
    // without expanding any neighbors; a mapped alias resolves to its canonical node.
    let mut stream = graph.execute(query(
        "MATCH (n:GraphNode) WHERE (n:Disease OR n:DiseaseEntity)
         AND any(value IN [n.uid, n.name, n.accession, n.id, n.db_id]
             WHERE replace(toLower(toString(value)), '_', ' ') CONTAINS toLower($q))
         WITH n, CASE WHEN any(value IN [n.uid, n.name, n.accession, n.id, n.db_id]
             WHERE replace(toLower(toString(value)), '_', ' ') = toLower($q)) THEN 0 ELSE 1 END AS rank
         ORDER BY rank, CASE WHEN n:Disease THEN 0 ELSE 1 END,
             size(coalesce(n.name, n.accession, n.uid)), n.uid LIMIT 1
         OPTIONAL MATCH (n)-[:MAPPED_VIA]->(:GraphNode)-[:MAPS_TO]->(canonical:GraphNode:Disease)
         WITH n, canonical ORDER BY canonical.uid LIMIT 1
         WITH coalesce(canonical, n) AS disease
         RETURN disease.uid AS id, labels(disease) AS labels, properties(disease) AS props"
    ).param("q", term.replace('_', " "))).await.map_err(graph_error)?;
    if let Some(row) = stream.next().await.map_err(graph_error)? {
        let node = RawNode {
            id: row.get("id").map_err(graph_error)?,
            labels: row.get("labels").map_err(graph_error)?,
            props: row.get("props").map_err(graph_error)?,
        };
        snapshot.nodes.push(view_node(&node, None));
    }
    Ok(snapshot)
}

fn view_node(node: &RawNode, canonical: Option<&RawNode>) -> ViewNode {
    ViewNode {
        id: node.id.clone(),
        label: label(node),
        kind: kind(node).into(),
        labels: node
            .labels
            .iter()
            .filter(|l| *l != "GraphNode")
            .cloned()
            .collect(),
        description: text(&node.props, "description").or_else(|| text(&node.props, "abstract")),
        links: source_links(node),
        community_url: (kind(node) == "disease")
            .then(|| format!("/community/{}", community_key(canonical.unwrap_or(node)))),
    }
}

fn project(
    nodes: &BTreeMap<String, RawNode>,
    edges: &[RawEdge],
    seeds: &BTreeSet<String>,
    limit: usize,
) -> (Vec<ViewNode>, Vec<ViewEdge>, bool) {
    let outgoing = |id: &str, kind: &str| {
        edges
            .iter()
            .filter(move |e| e.source == id && e.kind == kind)
            .map(|e| e.target.clone())
            .collect::<Vec<_>>()
    };
    let incoming = |id: &str, kind: &str| {
        edges
            .iter()
            .filter(move |e| e.target == id && e.kind == kind)
            .map(|e| e.source.clone())
            .collect::<Vec<_>>()
    };
    let mut projected = Vec::new();
    for edge in edges {
        if nodes.get(&edge.source).is_some_and(display)
            && nodes.get(&edge.target).is_some_and(display)
            && edge.kind != "IN_SNAPSHOT"
        {
            projected.push(ViewEdge {
                id: edge.id.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                label: text(&edge.props, "relation_type")
                    .unwrap_or_else(|| edge.kind.to_lowercase().replace('_', " ")),
                kind: edge.kind.clone(),
                evidence_id: text(&edge.props, "summary_uid").unwrap_or_else(|| edge.id.clone()),
                evidence_kind: if edge.kind == "PUBTATOR_RELATION" {
                    "summary"
                } else {
                    "direct"
                }
                .into(),
                reported_papers: edge.props.get("publications").and_then(Value::as_u64),
                context: edge.props.clone(),
            });
        }
        if [
            "HAS_PHENOTYPE",
            "EXCLUDES_PHENOTYPE",
            "CONFLICTING_PHENOTYPE",
        ]
        .contains(&edge.kind.as_str())
        {
            for disease in outgoing(&edge.source, "PROFILE_OF") {
                projected.push(ViewEdge {
                    id: edge.id.clone(),
                    source: disease,
                    target: edge.target.clone(),
                    label: edge.kind.to_lowercase().replace('_', " "),
                    kind: edge.kind.clone(),
                    evidence_id: edge.id.clone(),
                    evidence_kind: "phenotype".into(),
                    reported_papers: None,
                    context: json!({"profile_uid":edge.source}),
                });
            }
        }
    }
    for node in nodes.values() {
        if has(node, "HpoAnnotation") {
            for profile in incoming(&node.id, "HAS_ANNOTATION") {
                for disease in outgoing(&profile, "PROFILE_OF") {
                    for publication in outgoing(&node.id, "SUPPORTED_BY") {
                        if !nodes
                            .get(&publication)
                            .is_some_and(|p| has(p, "Publication"))
                        {
                            continue;
                        }
                        projected.push(ViewEdge {
                            id: format!("{}:{publication}:{disease}", node.id),
                            source: publication.clone(),
                            target: disease.clone(),
                            label: "cited by HPO annotation".into(),
                            kind: "ANNOTATION_CITATION".into(),
                            evidence_id: publication,
                            evidence_kind: "publication".into(),
                            reported_papers: None,
                            context: json!({"annotation_uid":node.id,"profile_uid":profile,"phenotypes":outgoing(&node.id,"PHENOTYPE"),"excluded":node.props.get("excluded")}),
                        });
                    }
                }
            }
        }
        if has(node, "ExtractedRelation") {
            let participants: Vec<_> = edges
                .iter()
                .filter(|e| e.source == node.id && e.kind == "HAS_PARTICIPANT")
                .collect();
            for source in participants
                .iter()
                .filter(|e| text(&e.props, "role").as_deref() == Some("role1"))
            {
                for target in participants
                    .iter()
                    .filter(|e| text(&e.props, "role").as_deref() == Some("role2"))
                {
                    projected.push(ViewEdge {
                        id: format!("{}:{}:{}", node.id, source.target, target.target),
                        source: source.target.clone(),
                        target: target.target.clone(),
                        label: text(&node.props, "type")
                            .unwrap_or_else(|| "extracted relation".into()),
                        kind: "EXTRACTED_RELATION".into(),
                        evidence_id: node.id.clone(),
                        evidence_kind: "extracted".into(),
                        reported_papers: None,
                        context: node.props.clone(),
                    });
                }
            }
        }
        if has(node, "DiseaseMapping") {
            for source in incoming(&node.id, "MAPPED_VIA") {
                for target in outgoing(&node.id, "MAPS_TO") {
                    projected.push(ViewEdge {
                        id: node.id.clone(),
                        source: source.clone(),
                        target,
                        label: "maps to disease".into(),
                        kind: "DISEASE_MAPPING".into(),
                        evidence_id: node.id.clone(),
                        evidence_kind: "mapping".into(),
                        reported_papers: None,
                        context: node.props.clone(),
                    });
                }
            }
        }
        if has(node, "Mention") {
            for doc in incoming(&node.id, "HAS_MENTION") {
                for publication in incoming(&doc, "HAS_DOCUMENT") {
                    for entity in outgoing(&node.id, "DENOTES") {
                        projected.push(ViewEdge {
                            id: format!("{}:{entity}", node.id),
                            source: publication.clone(),
                            target: entity,
                            label: "mentions".into(),
                            kind: "MENTIONS".into(),
                            evidence_id: publication.clone(),
                            evidence_kind: "publication".into(),
                            reported_papers: None,
                            context: json!({"mention_uid":node.id}),
                        });
                    }
                }
            }
        }
    }
    // Give matching/connected domain nodes priority over disconnected publications.
    let connected: BTreeSet<_> = projected
        .iter()
        .flat_map(|e| [&e.source, &e.target])
        .collect();
    let mut visible: Vec<_> = nodes.values().filter(|n| display(n)).collect();
    visible.sort_by_key(|n| {
        (
            !seeds.contains(&n.id),
            !connected.contains(&n.id),
            kind(n) == "publication",
            n.id.clone(),
        )
    });
    let clipped = visible.len() > limit;
    visible.truncate(limit);
    let ids: BTreeSet<_> = visible.iter().map(|n| n.id.clone()).collect();
    projected.retain(|e| ids.contains(&e.source) && ids.contains(&e.target));
    let view_nodes = visible
        .into_iter()
        .map(|n| {
            let canonical = if kind(n) == "disease" {
                outgoing(&n.id, "MAPPED_VIA")
                    .iter()
                    .flat_map(|m| outgoing(m, "MAPS_TO"))
                    .find_map(|id| nodes.get(&id).cloned())
            } else {
                None
            };
            view_node(n, canonical.as_ref())
        })
        .collect();
    (view_nodes, projected, clipped)
}

#[derive(Deserialize)]
pub struct EvidenceQuery {
    pub id: String,
    pub kind: String,
    pub offset: Option<i64>,
}
#[derive(Serialize)]
pub struct Evidence {
    pub papers: Vec<ViewNode>,
    pub has_more: bool,
    pub offset: i64,
    pub note: String,
}
pub async fn evidence(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(input): Query<EvidenceQuery>,
) -> Result<Json<Evidence>> {
    if input.id.len() > 1000 || input.offset.unwrap_or(0) < 0 || input.offset.unwrap_or(0) > 10000 {
        return Err(AppError::bad_request("invalid evidence query"));
    }
    let pattern = match input.kind.as_str() {
        "phenotype" => {
            "MATCH (profile:GraphNode)-[r]->(term:GraphNode) WHERE r.uid = $id AND type(r) IN ['HAS_PHENOTYPE','EXCLUDES_PHENOTYPE','CONFLICTING_PHENOTYPE'] MATCH (profile)-[:HAS_ANNOTATION]->(annotation:GraphNode)-[:PHENOTYPE]->(term) WHERE (type(r) = 'HAS_PHENOTYPE' AND coalesce(annotation.excluded, false) = false) OR (type(r) = 'EXCLUDES_PHENOTYPE' AND annotation.excluded = true) OR type(r) = 'CONFLICTING_PHENOTYPE' MATCH (annotation)-[:SUPPORTED_BY]->(p:GraphNode:Publication)"
        }
        "extracted" => {
            "MATCH (p:GraphNode:Publication)-[:HAS_DOCUMENT]->(:GraphNode)-[:HAS_RELATION]->(:GraphNode {uid: $id})"
        }
        "publication" => "MATCH (p:GraphNode:Publication {uid: $id})",
        "node" => {
            "MATCH (n:GraphNode {uid: $id}) CALL { WITH n MATCH (p:GraphNode:Publication) WHERE p = n RETURN p UNION WITH n MATCH (p:GraphNode:Publication)-[:HAS_DOCUMENT]->(:GraphNode)-[:HAS_MENTION]->(:GraphNode)-[:DENOTES]->(n) RETURN p UNION WITH n MATCH (profile:GraphNode)-[:PROFILE_OF]->(n) MATCH (profile)-[:HAS_ANNOTATION]->(:GraphNode)-[:SUPPORTED_BY]->(p:GraphNode:Publication) RETURN p UNION WITH n MATCH (annotation:GraphNode)-[:PHENOTYPE]->(n) MATCH (annotation)-[:SUPPORTED_BY]->(p:GraphNode:Publication) RETURN p UNION WITH n MATCH (p:GraphNode:Publication)-[:HAS_DOCUMENT]->(:GraphNode)-[:HAS_RELATION]->(:GraphNode)-[:HAS_PARTICIPANT]->(n) RETURN p UNION WITH n MATCH (n)-[:SUPPORTED_BY]->(p:GraphNode:Publication) RETURN p UNION WITH n MATCH (n)-[:MAPPED_VIA]->(:GraphNode)-[:MAPS_TO]->(d:GraphNode) MATCH (profile:GraphNode)-[:PROFILE_OF]->(d) MATCH (profile)-[:HAS_ANNOTATION]->(:GraphNode)-[:SUPPORTED_BY]->(p:GraphNode:Publication) RETURN p UNION WITH n MATCH (profile:GraphNode)-[:IN_SNAPSHOT]->(n) MATCH (profile)-[:HAS_ANNOTATION]->(:GraphNode)-[:SUPPORTED_BY]->(p:GraphNode:Publication) RETURN p }"
        }
        "summary" | "direct" | "mapping" => {
            "MATCH (:GraphNode {uid: $id})-[:SUPPORTED_BY]->(p:GraphNode:Publication)"
        }
        _ => return Err(AppError::bad_request("invalid evidence kind")),
    };
    let read = async {
        let graph = state.graph.connection().await?;
        let cypher = format!(
            "{pattern} RETURN DISTINCT p.uid AS id, labels(p) AS labels, properties(p) AS props ORDER BY p.uid SKIP $offset LIMIT 21"
        );
        let mut stream = graph
            .execute(
                query(&cypher)
                    .param("id", input.id)
                    .param("offset", input.offset.unwrap_or(0)),
            )
            .await
            .map_err(graph_error)?;
        let mut papers = Vec::new();
        while let Some(row) = stream.next().await.map_err(graph_error)? {
            let node = RawNode {
                id: row.get("id").map_err(graph_error)?,
                labels: row.get("labels").map_err(graph_error)?,
                props: row.get("props").map_err(graph_error)?,
            };
            papers.push(ViewNode {
                id: node.id.clone(),
                label: label(&node),
                kind: "publication".into(),
                labels: node.labels.clone(),
                description: text(&node.props, "abstract"),
                links: source_links(&node),
                community_url: None,
            });
        }
        let has_more = papers.len() > 20;
        papers.truncate(20);
        let note = match input.kind.as_str() {
            "node" => {
                "Stored literature linked to this source. Mentions are context, not proof of an association."
            }
            "summary" => {
                "PubTator summaries can contain a paper count without individual citations. Only explicitly stored supporting papers are listed here."
            }
            "direct" => {
                "Only explicitly stored citations are shown. An ontology hierarchy or identifier link is not a paper-supported biological association."
            }
            "mapping" => {
                "This is identifier/name mapping evidence, not a paper-supported biological association."
            }
            "phenotype" => {
                "These papers are cited by the matching HPO annotation rows for this phenotype and profile."
            }
            "extracted" => {
                "These papers contain the original extracted relation. An extracted relation is not independently verified causality."
            }
            _ => {
                "The original stored publication. Citations and mentions provide context; association edges show their specific supporting papers."
            }
        };
        Ok(Evidence {
            papers,
            has_more,
            offset: input.offset.unwrap_or(0),
            note: note.into(),
        })
    };
    tokio::time::timeout(Duration::from_secs(15), read)
        .await
        .map_err(|_| graph_error("evidence read timed out"))?
        .map(Json)
}
