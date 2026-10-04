use crate::{Error, GraphBatch, Result, model::Properties};
use neo4rs::{BoltType, Graph, Query, query};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
pub struct Store {
    graph: Graph,
    cache_dir: Arc<PathBuf>,
}

impl Store {
    /// Connect using neo4rs configuration (database, credentials, pool and TLS settings).
    pub async fn connect(config: neo4rs::Config) -> Result<Self> {
        Self::from_graph(Graph::connect(config).await?).await
    }
    pub async fn connect_with_cache_dir(
        config: neo4rs::Config,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::from_graph_with_cache_dir(Graph::connect(config).await?, cache_dir).await
    }
    pub async fn from_graph(graph: Graph) -> Result<Self> {
        let directory = std::env::var_os("HERDLINK_QUERY_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".graph-query-cache"));
        Self::from_graph_with_cache_dir(graph, directory).await
    }
    pub async fn from_graph_with_cache_dir(
        graph: Graph,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        tokio::fs::create_dir_all(cache_dir.as_ref()).await?;
        let cache_dir = Arc::new(tokio::fs::canonicalize(cache_dir.as_ref()).await?);
        let store = Self { graph, cache_dir };
        store.graph.run(query("CREATE CONSTRAINT graph_node_uid IF NOT EXISTS FOR (n:GraphNode) REQUIRE n.uid IS UNIQUE")).await?;
        store.graph.run(query("CREATE INDEX query_cache_expiry IF NOT EXISTS FOR (n:QueryCache) ON (n.expires_at)")).await?;
        Ok(store)
    }
    /// Escape hatch for application-specific queries and future migrations.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Immutable raw response files are installed before their graph references commit.
    pub(crate) async fn write_payload(&self, payload: &str) -> Result<String> {
        let digest = format!("{:x}", Sha256::digest(payload.as_bytes()));
        let bytes = payload.as_bytes().to_vec();
        let path = self.cache_dir.join(format!("{digest}.json"));
        let directory = self.cache_dir.clone();
        tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            let mut temporary = tempfile::NamedTempFile::new_in(directory.as_ref())?;
            temporary.write_all(&bytes)?;
            temporary.as_file().sync_all()?;
            match temporary.persist_noclobber(path) {
                Ok(_) => Ok(()),
                Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                Err(e) => Err(e.error),
            }
        })
        .await??;
        Ok(digest)
    }

    pub async fn upsert(&self, batch: &GraphBatch) -> Result<()> {
        self.write(batch, None).await
    }
    /// Optional complete import. Each bounded transaction is independently resumable via MERGE.
    /// Read-through clients normally persist only requested profiles and ancestor subgraphs.
    pub async fn import_hpo(&self, dataset: &pubtator3_hpo::Dataset) -> Result<()> {
        let terms: Vec<_> = dataset.ontology_records().keys().cloned().collect();
        for chunk in terms.chunks(100) {
            self.upsert(&crate::ontology_terms(dataset, chunk.iter().cloned()))
                .await?;
        }
        let mut batch = GraphBatch::default();
        for (index, profile) in dataset.profiles().enumerate() {
            batch.extend(crate::profile(profile, dataset));
            if (index + 1) % 25 == 0 {
                self.upsert(&batch).await?;
                batch = GraphBatch::default();
            }
        }
        if !batch.nodes.is_empty() {
            self.upsert(&batch).await?;
        }
        Ok(())
    }
    pub(crate) async fn cached<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut stream = self.graph.execute(query(
            "MATCH (c:GraphNode:QueryCache {uid: $key}) WHERE c.expires_at > timestamp() RETURN c.payload_hash AS hash"
        ).param("key", key)).await?;
        let Some(row) = stream.next().await? else {
            return Ok(None);
        };
        let digest: String = row.get("hash")?;
        if digest.len() != 64 || !digest.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::Invalid("invalid cache payload hash".into()));
        }
        let payload =
            match tokio::fs::read_to_string(self.cache_dir.join(format!("{digest}.json"))).await {
                Ok(payload) => payload,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
            };
        if format!("{:x}", Sha256::digest(payload.as_bytes())) != digest {
            return Err(Error::Invalid("cache payload checksum mismatch".into()));
        }
        Ok(Some(serde_json::from_str(&payload)?))
    }
    /// Evict a query response. Domain/evidence nodes remain available for graph queries.
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        self.graph
            .run(
                query("MATCH (c:GraphNode:QueryCache {uid: $key}) DETACH DELETE c")
                    .param("key", key),
            )
            .await?;
        Ok(())
    }
    /// Remove expired response payloads without deleting biomedical objects.
    pub async fn prune_cache(&self) -> Result<()> {
        self.graph
            .run(query(
                "MATCH (c:QueryCache) WHERE c.expires_at <= timestamp() DETACH DELETE c",
            ))
            .await?;
        Ok(())
    }

    pub(crate) async fn cache_write(&self, batch: &GraphBatch, key: &str) -> Result<()> {
        self.write(batch, Some(key)).await
    }
    async fn write(&self, batch: &GraphBatch, replace_cache: Option<&str>) -> Result<()> {
        batch.validate()?;
        let mut queries = Vec::<Query>::new();
        // Parameterize all data; the only generated syntax consists of validated identifiers.
        let mut groups = BTreeMap::<String, Vec<HashMap<String, BoltType>>>::new();
        for node in batch.nodes.values() {
            let mut labels = node.labels.clone();
            labels.sort();
            labels.dedup();
            groups
                .entry(labels.join(":"))
                .or_default()
                .push(HashMap::from([
                    ("uid".into(), node.uid.clone().into()),
                    ("props".into(), bolt_props(&node.properties).into()),
                ]));
        }
        for (labels, rows) in groups {
            queries.push(query(&format!("UNWIND $rows AS row MERGE (n:GraphNode {{uid: row.uid}}) SET n:{labels} SET n += row.props" )).param("rows", rows));
        }
        if let Some(key) = replace_cache {
            queries.push(query("MATCH (c:GraphNode:QueryCache {uid: $key}) SET c.cached_at = timestamp(), c.expires_at = timestamp() + c.ttl_ms").param("key", key));
            queries.push(
                query("MATCH (c:GraphNode:QueryCache {uid: $key})-[r:CACHED_RESULT]->() DELETE r")
                    .param("key", key),
            );
        }
        let mut groups = BTreeMap::<String, Vec<HashMap<String, BoltType>>>::new();
        for edge in batch.edges.values() {
            groups
                .entry(edge.kind.clone())
                .or_default()
                .push(HashMap::from([
                    ("uid".into(), edge.uid.clone().into()),
                    ("source".into(), edge.source.clone().into()),
                    ("target".into(), edge.target.clone().into()),
                    ("props".into(), bolt_props(&edge.properties).into()),
                ]));
        }
        for (kind, rows) in groups {
            queries.push(query(&format!("UNWIND $rows AS row MATCH (a:GraphNode {{uid: row.source}}), (b:GraphNode {{uid: row.target}}) MERGE (a)-[r:{kind} {{uid: row.uid}}]->(b) SET r += row.props")).param("rows", rows));
        }
        let mut tx = self.graph.start_txn().await?;
        if let Err(error) = tx.run_queries(queries).await {
            tx.rollback().await?;
            return Err(error.into());
        }
        tx.commit().await?;
        Ok(())
    }
}

fn bolt_props(properties: &Properties) -> HashMap<String, BoltType> {
    properties
        .iter()
        .filter_map(|(key, value)| {
            // Missing metadata must not erase known names from earlier, richer responses.
            if value.is_null() {
                return None;
            }
            let value: BoltType = match value {
                Value::Bool(v) => (*v).into(),
                Value::Number(v) if v.as_i64().is_some() => v.as_i64().unwrap().into(),
                Value::Number(v) if v.is_u64() => v.to_string().into(),
                Value::Number(v) if v.as_f64().is_some() => v.as_f64().unwrap().into(),
                Value::String(v) => v.clone().into(),
                // Neo4j cannot store nested maps or mixed lists as properties. Retain losslessly as JSON.
                v => serde_json::to_string(v)
                    .expect("serializable metadata")
                    .into(),
            };
            Some((key.clone(), value))
        })
        .collect()
}
