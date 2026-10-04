use std::{sync::Arc, time::Duration};

use reqwest::{Url, header::RETRY_AFTER};
use serde::de::DeserializeOwned;
use tokio::{sync::Mutex, time::Instant};

use crate::{
    AutocompleteRequest, Document, Entity, Error, ExportFormat, MeshDescriptorId, MeshSynonyms,
    Page, Pmcid, Pmid, RelatedEntity, RelationsRequest, Result, SearchQuery, SearchResponse,
    TextScope, parse_documents,
};

pub const DEFAULT_BASE_URL: &str = "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/";
pub const DEFAULT_MESH_BASE_URL: &str = "https://id.nlm.nih.gov/mesh/";
/// Conservative GET batch size used by this client. Larger inputs can use `annotations_batched`.
pub const MAX_EXPORT_IDS: usize = 100;

/// Reusable async client. Clones share the HTTP pool and request pacing.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base: Url,
    mesh_base: Url,
    timeout: Duration,
    interval: Duration,
    next_request: Arc<Mutex<Instant>>,
}

#[derive(Debug)]
pub struct ClientBuilder {
    http: Option<reqwest::Client>,
    base: String,
    mesh_base: String,
    timeout: Duration,
    interval: Duration,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            http: None,
            base: DEFAULT_BASE_URL.into(),
            mesh_base: DEFAULT_MESH_BASE_URL.into(),
            timeout: Duration::from_secs(60),
            interval: Duration::from_millis(350),
        }
    }
}

impl ClientBuilder {
    /// Override the API root for a proxy or local test server.
    pub fn base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }
    /// Override the companion MeSH API root for a proxy or local test server.
    pub fn mesh_base_url(mut self, base: impl Into<String>) -> Self {
        self.mesh_base = base.into();
        self
    }
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    /// Default: 350 ms between request starts, shared across clones. Zero disables pacing.
    pub fn request_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> Result<Client> {
        let base = api_root(&self.base)?;
        let mesh_base = api_root(&self.mesh_base)?;
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder()
                .user_agent(concat!("pubtator3-rust/", env!("CARGO_PKG_VERSION")))
                .build()?,
        };
        Ok(Client {
            http,
            base,
            mesh_base,
            timeout: self.timeout,
            interval: self.interval,
            next_request: Arc::new(Mutex::new(Instant::now())),
        })
    }
}

impl Client {
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// API roots identify the upstream for persistent response caches.
    pub fn cache_identity(&self) -> String {
        format!("{}|{}", self.base, self.mesh_base)
    }

    /// List MeSH names/entry terms for an entity selected from autocomplete.
    /// Rejects entities without an `ncbi_mesh` descriptor ID before sending a request.
    pub async fn synonyms(&self, entity: &Entity) -> Result<MeshSynonyms> {
        self.mesh_synonyms(&entity.mesh_descriptor_id()?).await
    }

    /// List MeSH terms directly by descriptor ID, without an autocomplete request.
    /// Uses NLM's companion API and shares this client's timeout, pool, and pacing.
    pub async fn mesh_synonyms(&self, descriptor: &MeshDescriptorId) -> Result<MeshSynonyms> {
        let body = self
            .get_text_at(
                &self.mesh_base,
                "lookup/details",
                &[
                    ("descriptor", descriptor.to_string()),
                    ("includes", "terms".into()),
                ],
            )
            .await?;
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn autocomplete(&self, request: &AutocompleteRequest) -> Result<Vec<Entity>> {
        if request.query.trim().is_empty() {
            return Err(Error::InvalidRequest(
                "autocomplete query cannot be empty".into(),
            ));
        }
        let mut params = vec![("query", request.query.clone())];
        if let Some(concept) = request.concept {
            params.push(("concept", concept.to_string()));
        }
        if let Some(limit) = request.limit {
            params.push(("limit", limit.to_string()));
        }
        self.get_json("entity/autocomplete/", &params).await
    }

    pub async fn relations(&self, request: &RelationsRequest) -> Result<Vec<RelatedEntity>> {
        let mut params = vec![("e1", request.entity.to_string())];
        if let Some(kind) = request.target_type {
            params.push(("e2", kind.to_string()));
        }
        if let Some(kind) = request.relation_type {
            params.push(("type", kind.to_string()));
        }
        if let Some(limit) = request.limit {
            params.push(("limit", limit.to_string()));
        }
        self.get_json("relations", &params).await
    }

    pub async fn search(&self, query: &SearchQuery, page: Page) -> Result<SearchResponse> {
        self.get_json(
            "search/",
            &[
                ("text", query.to_string()),
                ("page", page.get().to_string()),
            ],
        )
        .await
    }

    pub async fn annotations(&self, pmids: &[Pmid], scope: TextScope) -> Result<Vec<Document>> {
        parse_documents(&self.export(pmids, ExportFormat::BioCJson, scope).await?)
    }

    /// Sequential, paced GET batches. An error discards the accumulated result;
    /// callers needing checkpoints should call `annotations` per chunk instead.
    pub async fn annotations_batched(
        &self,
        pmids: &[Pmid],
        scope: TextScope,
    ) -> Result<Vec<Document>> {
        if pmids.is_empty() {
            return Err(Error::InvalidRequest("provide at least one PMID".into()));
        }
        let mut documents = Vec::new();
        for batch in pmids.chunks(MAX_EXPORT_IDS) {
            documents.extend(self.annotations(batch, scope).await?);
        }
        Ok(documents)
    }

    /// Raw BioC JSON/XML or PubTator output. Full text requires a BioC format.
    pub async fn export(
        &self,
        pmids: &[Pmid],
        format: ExportFormat,
        scope: TextScope,
    ) -> Result<String> {
        check_batch(pmids.len())?;
        check_format(format, scope)?;
        let mut params = vec![("pmids", join_ids(pmids))];
        if scope == TextScope::FullText {
            params.push(("full", "true".into()));
        }
        self.get_text(&format!("publications/export/{format}"), &params)
            .await
    }

    pub async fn pmc_annotations(&self, pmcids: &[Pmcid]) -> Result<Vec<Document>> {
        parse_documents(&self.pmc_export(pmcids, ExportFormat::BioCJson).await?)
    }

    pub async fn pmc_export(&self, pmcids: &[Pmcid], format: ExportFormat) -> Result<String> {
        check_batch(pmcids.len())?;
        check_format(format, TextScope::FullText)?;
        self.get_text(
            &format!("publications/pmc_export/{format}"),
            &[("pmcids", join_ids(pmcids))],
        )
        .await
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        Ok(serde_json::from_str(
            &self.get_text(endpoint, params).await?,
        )?)
    }

    async fn get_text(&self, endpoint: &str, params: &[(&str, String)]) -> Result<String> {
        self.get_text_at(&self.base, endpoint, params).await
    }

    async fn get_text_at(
        &self,
        base: &Url,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> Result<String> {
        let url = base
            .join(endpoint)
            .map_err(|e| Error::InvalidRequest(e.to_string()))?;
        let request = self.http.get(url).query(params).timeout(self.timeout);
        {
            let mut next = self.next_request.lock().await;
            tokio::time::sleep_until(*next).await;
            *next = Instant::now() + self.interval;
        }
        let response = request.send().await?;
        let status = response.status();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let body = response.text().await?;
        if !status.is_success() {
            return Err(Error::Http {
                status,
                body: body.chars().take(2048).collect(),
                retry_after,
            });
        }
        Ok(body)
    }
}

fn api_root(value: &str) -> Result<Url> {
    let mut base =
        Url::parse(value).map_err(|e| Error::InvalidRequest(format!("invalid API root: {e}")))?;
    if !matches!(base.scheme(), "http" | "https")
        || base.host_str().is_none()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(Error::InvalidRequest(
            "API root must be an HTTP(S) URL without a query or fragment".into(),
        ));
    }
    if !base.path().ends_with('/') {
        base.set_path(&format!("{}/", base.path()));
    }
    Ok(base)
}

fn join_ids(ids: &[impl std::fmt::Display]) -> String {
    ids.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn check_batch(len: usize) -> Result<()> {
    if !(1..=MAX_EXPORT_IDS).contains(&len) {
        return Err(Error::InvalidRequest(format!(
            "export requires 1..={MAX_EXPORT_IDS} IDs; received {len}"
        )));
    }
    Ok(())
}

fn check_format(format: ExportFormat, scope: TextScope) -> Result<()> {
    if scope == TextScope::FullText && format == ExportFormat::PubTator {
        return Err(Error::InvalidRequest(
            "full-text export requires BioC JSON or XML".into(),
        ));
    }
    Ok(())
}
