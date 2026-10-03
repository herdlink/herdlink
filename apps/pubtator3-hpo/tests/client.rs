use pubtator3::{EntityId, Page};
use pubtator3_hpo::{Client, Dataset, Error, MappingMethod, SimilarityOptions};
use reqwest::Url;
use serde_json::{Value, json};
use std::{io::Cursor, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};

fn dataset() -> Dataset {
    Dataset::from_readers(
        Cursor::new(include_str!("fixtures/hp.obo")),
        include_bytes!("fixtures/phenotype.hpoa").as_slice(),
        include_bytes!("fixtures/mondo.json").as_slice(),
    )
    .unwrap()
}

fn entity(id: &str, name: &str, mesh: &str) -> Value {
    json!({"_id": id, "name": name, "biotype": "disease", "db": "ncbi_mesh", "db_id": mesh})
}

async fn server(
    responses: Vec<Value>,
) -> (Client, mpsc::Receiver<Url>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel(16);
    let task = tokio::spawn(async move {
        for body in responses {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut bytes = [0; 2048];
                let n = socket.read(&mut bytes).await.unwrap();
                assert_ne!(n, 0);
                request.extend_from_slice(&bytes[..n]);
                if request.windows(4).any(|p| p == b"\r\n\r\n") {
                    break;
                }
            }
            let request = std::str::from_utf8(&request).unwrap();
            let path = request
                .lines()
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap();
            tx.send(Url::parse(&format!("http://{address}{path}")).unwrap())
                .await
                .unwrap();
            let body = body.to_string();
            socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
        }
    });
    let pubtator = pubtator3::Client::builder()
        .base_url(format!("http://{address}/api"))
        .http_client(reqwest::Client::builder().no_proxy().build().unwrap())
        .request_interval(Duration::ZERO)
        .build()
        .unwrap();
    (Client::new(pubtator, dataset()), rx, task)
}

fn parameter(url: &Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}

#[tokio::test]
async fn wrapper_resolves_maps_and_ranks_without_per_candidate_requests() {
    let source = entity("@DISEASE_Source", "Source Disease", "D000001");
    let similar_name = entity("@DISEASE_Source_like", "Source Disease-like", "D000003");
    let (client, mut requests, task) = server(vec![json!([source, similar_name])]).await;
    let report = client
        .similar_diseases("Source Disease", None, &SimilarityOptions::default())
        .await
        .unwrap();
    assert_eq!(report.mapping.method, MappingMethod::ExactMesh);
    assert_eq!(report.source.id.as_str(), "MONDO:0000001");
    assert_eq!(report.matches[0].disease_id.as_str(), "MONDO:0000002");
    assert_eq!(report.corpus_diseases, 7);
    let url = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/entity/autocomplete/");
    assert_eq!(parameter(&url, "query").as_deref(), Some("Source Disease"));
    assert_eq!(parameter(&url, "concept").as_deref(), Some("disease"));
    task.await.unwrap();
    assert!(requests.try_recv().is_err());
}

#[tokio::test]
async fn ambiguous_autocomplete_requires_explicit_entity_and_rejects_missing_selection() {
    let candidates = json!([
        entity("@DISEASE_Source", "Source Disease", "D000001"),
        entity("@DISEASE_Twin", "Twin Disease", "D000002")
    ]);
    let (client, _requests, task) =
        server(vec![candidates.clone(), candidates.clone(), candidates]).await;
    assert!(matches!(client.resolve_disease("Disease", None).await,
        Err(Error::AmbiguousDisease { candidates, .. }) if candidates.len() == 2));
    let selected: EntityId = "@DISEASE_Source".parse().unwrap();
    assert_eq!(
        client
            .disease_phenotypes("Disease", Some(&selected))
            .await
            .unwrap()
            .entity
            .id,
        selected
    );
    assert!(matches!(
        client
            .resolve_disease("Disease", Some(&"@DISEASE_Missing".parse().unwrap()))
            .await,
        Err(Error::NoDisease(_))
    ));
    task.await.unwrap();
}

#[tokio::test]
async fn query_can_supply_an_exact_alias_but_strict_ids_refuse_it() {
    let candidates = json!([entity("@DISEASE_Test", "Unmapped label", "C000000009")]);
    let (client, _requests, task) = server(vec![candidates.clone(), candidates]).await;
    let mapped = client
        .disease_phenotypes("Source alias", None)
        .await
        .unwrap();
    assert_eq!(mapped.mapping.method, MappingMethod::ExactName);
    assert!(matches!(
        client
            .allow_name_fallback(false)
            .disease_phenotypes("Source alias", None)
            .await,
        Err(Error::Unmapped(_))
    ));
    task.await.unwrap();
}

#[tokio::test]
async fn reverse_mapping_prefers_mesh_and_never_overrides_a_conflicting_known_id() {
    let candidates = json!([
        entity("@DISEASE_Twin", "Different lexical label", "D000002"),
        entity("@DISEASE_Lexical", "Twin Disease", "C000000009"),
        entity("@DISEASE_Wrong", "Twin Disease", "D000003")
    ]);
    let (client, _requests, task) = server(vec![candidates]).await;
    let disease = client
        .dataset()
        .similar(&"OMIM:100001".parse().unwrap(), &Default::default())
        .unwrap()
        .remove(0);
    let matches = client.pubtator_entities(&disease).await.unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].entity.id.as_str(), "@DISEASE_Twin");
    assert_eq!(matches[0].method, MappingMethod::ExactMesh);
    task.await.unwrap();
    let (client, _requests, task) = server(vec![json!([entity(
        "@DISEASE_Wrong",
        "Twin Disease",
        "D000003"
    )])])
    .await;
    assert!(client.pubtator_entities(&disease).await.unwrap().is_empty());
    task.await.unwrap();
}

#[tokio::test]
async fn reverse_ambiguity_stays_unmapped_until_an_override_is_supplied() {
    let candidates = json!([entity("@DISEASE_Shared", "Shared Disease", "D000099")]);
    let (client, _requests, task) = server(vec![candidates.clone(), candidates]).await;
    let disease = client
        .dataset()
        .similar(&"OMIM:100004".parse().unwrap(), &Default::default())
        .unwrap()
        .remove(0);
    assert_eq!(disease.disease_id.as_str(), "MONDO:0000006");
    assert!(client.pubtator_entities(&disease).await.unwrap().is_empty());
    let client = client
        .with_override("D000099".parse().unwrap(), "OMIM:100006".parse().unwrap())
        .unwrap();
    let matches = client.pubtator_entities(&disease).await.unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].method, MappingMethod::Override);
    task.await.unwrap();
}

#[tokio::test]
async fn reverse_mapping_tries_exact_synonyms_when_the_label_returns_nothing() {
    let (client, mut requests, task) = server(vec![
        json!([]),
        json!([entity("@DISEASE_Source", "PubTator label", "D000001")]),
    ])
    .await;
    let source = client
        .dataset()
        .similar(&"OMIM:100002".parse().unwrap(), &Default::default())
        .unwrap()
        .remove(0);
    assert_eq!(source.disease_id.as_str(), "MONDO:0000001");
    let matches = client.pubtator_entities(&source).await.unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].method, MappingMethod::ExactMesh);
    assert_eq!(
        parameter(&requests.recv().await.unwrap(), "query").as_deref(),
        Some("Source Disease")
    );
    assert_eq!(
        parameter(&requests.recv().await.unwrap(), "query").as_deref(),
        Some("Source alias")
    );
    task.await.unwrap();
}

#[tokio::test]
async fn paper_lookup_uses_entity_co_mentions_and_validates_entity_types() {
    let response: Value =
        serde_json::from_str(include_str!("../../pubtator3/tests/fixtures/search.json")).unwrap();
    let (client, mut requests, task) = server(vec![response]).await;
    let source: EntityId = "@DISEASE_Source".parse().unwrap();
    let target: EntityId = "@DISEASE_Twin".parse().unwrap();
    assert!(
        client
            .supporting_papers(&source, &"@GENE_Test".parse().unwrap(), Page::FIRST)
            .await
            .is_err()
    );
    let papers = client
        .supporting_papers(&source, &target, Page::FIRST)
        .await
        .unwrap();
    assert!(!papers.results.is_empty());
    let url = requests.recv().await.unwrap();
    assert_eq!(
        parameter(&url, "text").as_deref(),
        Some("(@DISEASE_Source) AND (@DISEASE_Twin)")
    );
    task.await.unwrap();
}
