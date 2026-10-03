use std::{num::NonZeroU32, time::Duration};

use pubtator3::*;
use reqwest::{StatusCode, Url};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};

const ENTITIES: &str = include_str!("fixtures/entities.json");
const RELATIONS: &str = include_str!("fixtures/relations.json");
const SEARCH: &str = include_str!("fixtures/search.json");
const ANNOTATIONS: &str = include_str!("fixtures/annotations.json");
const FULLTEXT: &str = include_str!("fixtures/fulltext.json");
const DISEASE: &str = include_str!("fixtures/disease.json");
const MESH_SYNONYMS: &str = include_str!("fixtures/mesh-als.json");

/// Minimal local HTTP fixture server; tests do not contact NCBI.
async fn server(
    responses: Vec<(u16, &'static str, &'static str)>,
) -> (
    Client,
    mpsc::Receiver<(Url, tokio::time::Instant)>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel(32);
    let task = tokio::spawn(async move {
        for (status, headers, body) in responses {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            loop {
                let mut buffer = [0; 4096];
                let n = socket.read(&mut buffer).await.unwrap();
                assert_ne!(n, 0, "request closed before headers");
                bytes.extend_from_slice(&buffer[..n]);
                if bytes.windows(4).any(|part| part == b"\r\n\r\n") {
                    break;
                }
            }
            let request = std::str::from_utf8(&bytes).unwrap();
            let mut line = request.lines().next().unwrap().split_whitespace();
            assert_eq!(line.next(), Some("GET"));
            let url = Url::parse(&format!("http://{address}{}", line.next().unwrap())).unwrap();
            tx.send((url, tokio::time::Instant::now())).await.unwrap();
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let client = Client::builder()
        .base_url(format!("http://{address}/api"))
        .mesh_base_url(format!("http://{address}/mesh"))
        .http_client(reqwest::Client::builder().no_proxy().build().unwrap())
        .request_interval(Duration::ZERO)
        .build()
        .unwrap();
    (client, rx, task)
}

fn param(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

#[test]
fn validated_identifiers_and_serde() {
    for bad in [
        "",
        "0",
        "-1",
        "+1",
        "1.5",
        " 123",
        "PMC123",
        "18446744073709551616",
    ] {
        assert!(bad.parse::<Pmid>().is_err(), "{bad}");
    }
    let pmid = Pmid::new(123).unwrap();
    assert_eq!(serde_json::from_str::<Pmid>("123").unwrap(), pmid);
    assert_eq!(serde_json::from_str::<Pmid>("\"123\"").unwrap(), pmid);
    assert_eq!(serde_json::to_string(&pmid).unwrap(), "123");
    assert!(serde_json::from_str::<Pmid>("0").is_err());
    assert!(serde_json::from_str::<Page>("0").is_err());
    assert!(Page::new(0).is_err());
    assert!(Page::new(u32::MAX).unwrap().next().is_none());
    for bad in ["123", "pmc123", "PMC0", "PMC-1", "PMC"] {
        assert!(bad.parse::<Pmcid>().is_err());
    }
    let pmcid: Pmcid = "PMC123".parse().unwrap();
    assert_eq!(
        serde_json::from_str::<Pmcid>(&serde_json::to_string(&pmcid).unwrap()).unwrap(),
        pmcid
    );
    assert!(serde_json::from_str::<EntityId>("\"@GENE_\"").is_err());
    for bad in [
        "GENE_BRAF",
        "@gene_BRAF",
        "@_BRAF",
        "@GENE_",
        "@GENE_BRAF|DISEASE",
        "@GENE_B RAF",
    ] {
        assert!(bad.parse::<EntityId>().is_err(), "{bad}");
    }
    let gene: EntityId = "@GENE_BRAF".parse().unwrap();
    assert_eq!(gene.namespace(), "GENE");
    // PubTator uses @ inside gene-cluster symbols, e.g. SAA@.
    let cluster: EntityId = "@GENE_SAA@".parse().unwrap();
    assert_eq!(cluster.namespace(), "GENE");
    assert_eq!(
        serde_json::from_str::<EntityId>("\"@GENE_SAA@\"").unwrap(),
        cluster
    );
    assert!("@CHEMICAL_MESH:C000606551".parse::<EntityId>().is_ok());
}

#[test]
fn captured_responses_and_bioc_variants() {
    let entities: Vec<Entity> = serde_json::from_str(ENTITIES).unwrap();
    assert_eq!(entities[0].id.as_str(), "@GENE_BRAF");
    assert_eq!(entities[0].db_id.as_deref(), Some("673"));
    let relations: Vec<RelatedEntity> = serde_json::from_str(RELATIONS).unwrap();
    assert_eq!(relations[0].relation_type, RelationType::Treat);
    assert!(relations[0].publications > 0);
    let mut search: SearchResponse = serde_json::from_str(SEARCH).unwrap();
    assert_eq!(search.results[0].pmid.get(), 37711410);
    assert_eq!(
        search.results[1].pmcid.as_ref().unwrap().as_str(),
        "PMC9910426"
    );
    assert_eq!(search.next_page().unwrap().get(), 2);
    search.current = Page::new(search.total_pages).unwrap();
    assert!(search.next_page().is_none());

    let documents = parse_documents(ANNOTATIONS).unwrap();
    let document = &documents[0];
    assert_eq!(document.pmid.unwrap().get(), 19894120);
    assert!(document.annotations().count() > 10);
    let gene = document
        .annotations()
        .find(|a| {
            a.infons
                .accession
                .as_ref()
                .is_some_and(|a| a.as_str() == "@GENE_ABCC1")
        })
        .unwrap();
    assert!(gene.infons.extra["normalized_id"].is_number());
    let relation = document.all_relations().next().unwrap();
    assert_eq!(
        relation
            .infons
            .role1
            .as_ref()
            .unwrap()
            .accession
            .as_ref()
            .unwrap()
            .as_str(),
        "@CHEMICAL_fluorexon"
    );
    assert_eq!(
        relation.infons.relation_type.as_deref(),
        Some("Positive_Correlation")
    );
    let single = serde_json::to_string(document).unwrap();
    for json in [
        format!("[{single}]"),
        format!("{{\"documents\":[{single}]}}"),
        single.clone(),
    ] {
        assert_eq!(parse_documents(&json).unwrap().len(), 1);
    }
    assert_eq!(
        parse_documents(&format!("{single}\n{single}\n"))
            .unwrap()
            .len(),
        2
    );
    assert!(parse_documents("{}").is_err());
    assert!(parse_documents("").is_err());
    assert!(parse_documents("{\"PubTator3\":[]}").unwrap().is_empty());
    assert_eq!(
        Location {
            offset: u64::MAX,
            length: 1
        }
        .end(),
        None
    );
}

#[test]
fn query_composition() {
    let drug: EntityId = "@CHEMICAL_Doxorubicin".parse().unwrap();
    let disease: EntityId = "@DISEASE_Neoplasms".parse().unwrap();
    assert_eq!(
        SearchQuery::relation(RelationFilter::Type(RelationType::Treat), &drug, &disease).as_str(),
        "relations:treat|@CHEMICAL_Doxorubicin|@DISEASE_Neoplasms"
    );
    assert_eq!(
        SearchQuery::relation_to_type(RelationFilter::Any, &drug, RelationEntityType::Disease)
            .as_str(),
        "relations:ANY|@CHEMICAL_Doxorubicin|DISEASE"
    );
    assert_eq!(
        SearchQuery::entity(&drug)
            .and(SearchQuery::entity(&disease))
            .as_str(),
        "(@CHEMICAL_Doxorubicin) AND (@DISEASE_Neoplasms)"
    );
    assert!(SearchQuery::text("  ").is_err());
}

#[test]
fn fulltext_ids_and_chemical_parentheses() {
    let documents = parse_documents(FULLTEXT).unwrap();
    let document = &documents[0];
    assert_eq!(document.id.as_str(), "6142073");
    assert_eq!(document.pmid.unwrap().get(), 29355051);
    assert_eq!(document.pmcid.as_ref().unwrap().as_str(), "PMC6142073");
    assert!(document.annotations().any(|annotation| {
        annotation.infons.accession.as_ref().is_some_and(|id| {
            id.as_str() == "@CHEMICAL_5_(6)_carboxyfluorescein_diacetate_succinimidyl_ester"
        })
    }));
    let invalid = FULLTEXT.replace(
        "@CHEMICAL_5_(6)_carboxyfluorescein_diacetate_succinimidyl_ester",
        "bad-id",
    );
    assert!(
        parse_documents(&invalid)
            .unwrap_err()
            .to_string()
            .contains("invalid EntityId")
    );
}

#[tokio::test]
async fn endpoints_and_query_encoding() {
    let (client, mut requests, task) = server(vec![
        (200, "", ENTITIES),
        (200, "", RELATIONS),
        (200, "", SEARCH),
        (200, "", ANNOTATIONS),
        (200, "", ANNOTATIONS),
        (200, "", ANNOTATIONS),
        (200, "", "<collection/>"),
        (200, "", "pubtator text"),
    ])
    .await;
    let lookup = AutocompleteRequest::new("BRAF & cancer+#")
        .concept(Concept::Gene)
        .limit(NonZeroU32::new(2).unwrap());
    assert_eq!(client.autocomplete(&lookup).await.unwrap().len(), 2);
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/entity/autocomplete/");
    assert_eq!(param(&url, "query").as_deref(), Some("BRAF & cancer+#"));
    assert_eq!(param(&url, "concept").as_deref(), Some("gene"));
    assert_eq!(param(&url, "limit").as_deref(), Some("2"));

    let request = RelationsRequest::new("@CHEMICAL_Doxorubicin".parse().unwrap())
        .relation_type(RelationType::Treat)
        .target_type(RelationEntityType::Disease);
    client.relations(&request).await.unwrap();
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/relations");
    assert_eq!(param(&url, "e1").as_deref(), Some("@CHEMICAL_Doxorubicin"));
    assert_eq!(param(&url, "type").as_deref(), Some("treat"));
    assert_eq!(param(&url, "e2").as_deref(), Some("disease"));

    let query = SearchQuery::relation_to_type(
        RelationFilter::Any,
        &request.entity,
        RelationEntityType::Disease,
    );
    client.search(&query, Page::new(2).unwrap()).await.unwrap();
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/search/");
    assert_eq!(param(&url, "text").as_deref(), Some(query.as_str()));
    assert_eq!(param(&url, "page").as_deref(), Some("2"));

    let ids = [Pmid::new(19894120).unwrap(), Pmid::new(29355051).unwrap()];
    client.annotations(&ids, TextScope::Abstract).await.unwrap();
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/publications/export/biocjson");
    assert_eq!(param(&url, "pmids").as_deref(), Some("19894120,29355051"));
    assert!(param(&url, "full").is_none());
    client.annotations(&ids, TextScope::FullText).await.unwrap();
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(param(&url, "full").as_deref(), Some("true"));
    assert!(param(&url, "full_text").is_none());

    client
        .pmc_annotations(&["PMC6142073".parse().unwrap()])
        .await
        .unwrap();
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/publications/pmc_export/biocjson");
    assert_eq!(param(&url, "pmcids").as_deref(), Some("PMC6142073"));
    assert_eq!(
        client
            .export(&ids, ExportFormat::BioCXml, TextScope::Abstract)
            .await
            .unwrap(),
        "<collection/>"
    );
    assert_eq!(
        requests.recv().await.unwrap().0.path(),
        "/api/publications/export/biocxml"
    );
    assert_eq!(
        client
            .export(&ids, ExportFormat::PubTator, TextScope::Abstract)
            .await
            .unwrap(),
        "pubtator text"
    );
    assert_eq!(
        requests.recv().await.unwrap().0.path(),
        "/api/publications/export/pubtator"
    );
    task.await.unwrap();
}

#[tokio::test]
async fn local_validation_and_http_errors() {
    let (client, mut requests, task) = server(vec![
        (429, "Retry-After: 5\r\n", "slow down"),
        (500, "", "server failed"),
        (200, "", "invalid json"),
    ])
    .await;
    assert!(matches!(
        client.annotations(&[], TextScope::Abstract).await,
        Err(Error::InvalidRequest(_))
    ));
    assert!(
        client
            .annotations(&vec![Pmid::new(1).unwrap(); 101], TextScope::Abstract)
            .await
            .is_err()
    );
    assert!(
        client
            .autocomplete(&AutocompleteRequest::new(" "))
            .await
            .is_err()
    );
    assert!(
        client
            .export(
                &[Pmid::new(1).unwrap()],
                ExportFormat::PubTator,
                TextScope::FullText
            )
            .await
            .is_err()
    );
    assert!(
        client
            .pmc_export(&["PMC1".parse().unwrap()], ExportFormat::PubTator)
            .await
            .is_err()
    );
    assert!(requests.try_recv().is_err());
    match client
        .search(&SearchQuery::text("test").unwrap(), Page::FIRST)
        .await
        .unwrap_err()
    {
        Error::Http {
            status,
            body,
            retry_after,
        } => {
            assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(body, "slow down");
            assert_eq!(retry_after.as_deref(), Some("5"));
        }
        error => panic!("unexpected {error}"),
    }
    requests.recv().await.unwrap();
    assert!(matches!(
        client.autocomplete(&AutocompleteRequest::new("x")).await,
        Err(Error::Http {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            ..
        })
    ));
    requests.recv().await.unwrap();
    assert!(matches!(
        client.autocomplete(&AutocompleteRequest::new("x")).await,
        Err(Error::Json(_))
    ));
    requests.recv().await.unwrap();
    task.await.unwrap();
}

#[tokio::test]
async fn batching_and_shared_pacing() {
    let (client, mut requests, task) =
        server(vec![(200, "", ANNOTATIONS), (200, "", ANNOTATIONS)]).await;
    let mut ids = vec![Pmid::new(1).unwrap(); 100];
    ids.push(Pmid::new(2).unwrap());
    assert_eq!(
        client
            .annotations_batched(&ids, TextScope::Abstract)
            .await
            .unwrap()
            .len(),
        2
    );
    let (first, _) = requests.recv().await.unwrap();
    let (second, _) = requests.recv().await.unwrap();
    assert_eq!(param(&first, "pmids").unwrap().split(',').count(), 100);
    assert_eq!(param(&second, "pmids").as_deref(), Some("2"));
    task.await.unwrap();

    let (fixture, mut requests, task) = server(vec![
        (200, "", ENTITIES),
        (200, "", ENTITIES),
        (200, "", ENTITIES),
    ])
    .await;
    // Obtain the fixture URL by sending one request, then use a separate server
    // with a paced client and its clone for the regression check.
    fixture
        .autocomplete(&AutocompleteRequest::new("BRAF"))
        .await
        .unwrap();
    let (url, _) = requests.recv().await.unwrap();
    let root = url.join("../..").unwrap();
    let paced = Client::builder()
        .base_url(root.as_str())
        .request_interval(Duration::from_millis(50))
        .http_client(reqwest::Client::builder().no_proxy().build().unwrap())
        .build()
        .unwrap();
    let clone = paced.clone();
    let request = AutocompleteRequest::new("BRAF");
    let start = tokio::time::Instant::now();
    let (first, second) = tokio::join!(paced.autocomplete(&request), clone.autocomplete(&request));
    first.unwrap();
    second.unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(50),
        "clones must share pacing"
    );
    requests.recv().await.unwrap();
    requests.recv().await.unwrap();
    task.await.unwrap();
}

#[tokio::test]
async fn request_timeout_is_a_transport_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
    });
    let client = Client::builder()
        .base_url(format!("http://{address}"))
        .timeout(Duration::from_millis(20))
        .http_client(reqwest::Client::builder().no_proxy().build().unwrap())
        .build()
        .unwrap();
    match client
        .autocomplete(&AutocompleteRequest::new("BRAF"))
        .await
        .unwrap_err()
    {
        Error::Transport(error) => assert!(error.is_timeout()),
        error => panic!("unexpected {error}"),
    }
    task.abort();
}

#[test]
fn invalid_api_roots() {
    for base in [
        "not a URL",
        "ftp://example.com",
        "https://example.com?query=x",
        "https://example.com#fragment",
    ] {
        assert!(Client::builder().base_url(base).build().is_err());
        assert!(Client::builder().mesh_base_url(base).build().is_err());
    }
}

#[test]
fn mesh_ids_and_synonym_metadata() {
    for id in ["D000690", "D000086382"] {
        let descriptor: MeshDescriptorId = id.parse().unwrap();
        assert_eq!(
            MeshDescriptorId::from_uri(&descriptor.to_uri()).unwrap(),
            descriptor
        );
        assert_eq!(
            serde_json::from_str::<MeshDescriptorId>(&format!("\"{id}\"")).unwrap(),
            descriptor
        );
    }
    for id in [
        "D123",
        "C000690",
        "M000690",
        "d000690",
        "D000000",
        "D000000000",
        "MESH:D000690",
        "D000690?x=1",
    ] {
        assert!(id.parse::<MeshDescriptorId>().is_err(), "{id}");
    }
    assert!(serde_json::from_str::<MeshDescriptorId>("\"D0\"").is_err());
    assert!(MeshDescriptorId::from_uri("https://example.com/mesh/D000690").is_err());
    assert!(MeshTermId::from_uri("http://id.nlm.nih.gov/mesh/D000690").is_err());

    let result: MeshSynonyms = serde_json::from_str(MESH_SYNONYMS).unwrap();
    assert_eq!(result.descriptor.as_str(), "D000690");
    assert_eq!(
        result.preferred_term().unwrap().label,
        "Amyotrophic Lateral Sclerosis"
    );
    assert_eq!(result.synonyms().count(), 14);
    assert!(
        result
            .synonyms()
            .any(|term| term.label == "Lou Gehrig's Disease")
    );
    assert!(!result.synonyms().any(|term| term.preferred == Some(true)));
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        serde_json::from_str::<serde_json::Value>(MESH_SYNONYMS).unwrap()
    );
    let unmarked = r#"{"descriptor":"http://id.nlm.nih.gov/mesh/D000690","terms":[{"resource":"http://id.nlm.nih.gov/mesh/T002090","label":"unmarked","new_field":1}]}"#;
    let result: MeshSynonyms = serde_json::from_str(unmarked).unwrap();
    assert!(result.preferred_term().is_none());
    assert_eq!(result.synonyms().count(), 0);
    assert_eq!(result.terms[0].extra["new_field"], 1);
}

#[tokio::test]
async fn normalize_and_list_synonyms_through_mesh_root() {
    let (client, mut requests, task) = server(vec![
        (200, "", DISEASE),
        (200, "", MESH_SYNONYMS),
        (200, "", MESH_SYNONYMS),
    ])
    .await;
    let entities = client
        .autocomplete(&AutocompleteRequest::new("Lou Gehrig disease").concept(Concept::Disease))
        .await
        .unwrap();
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/api/entity/autocomplete/");
    assert_eq!(
        entities[0].mesh_descriptor_id().unwrap().as_str(),
        "D000690"
    );
    let terms = client.synonyms(&entities[0]).await.unwrap();
    assert_eq!(terms.synonyms().count(), 14);
    let (url, _) = requests.recv().await.unwrap();
    assert_eq!(url.path(), "/mesh/lookup/details");
    assert_eq!(param(&url, "descriptor").as_deref(), Some("D000690"));
    assert_eq!(param(&url, "includes").as_deref(), Some("terms"));

    let descriptor: MeshDescriptorId = "D000690".parse().unwrap();
    client.mesh_synonyms(&descriptor).await.unwrap();
    assert_eq!(
        requests.recv().await.unwrap().0.path(),
        "/mesh/lookup/details"
    );
    task.await.unwrap();
}

#[tokio::test]
async fn unsupported_synonym_inputs_do_not_send_requests() {
    let (client, mut requests, task) = server(vec![(200, "", MESH_SYNONYMS)]).await;
    let gene: Vec<Entity> = serde_json::from_str(ENTITIES).unwrap();
    assert!(matches!(
        client.synonyms(&gene[0]).await,
        Err(Error::UnsupportedSynonyms { .. })
    ));
    let mut disease: Entity = serde_json::from_str::<Vec<Entity>>(DISEASE)
        .unwrap()
        .remove(0);
    disease.db_id = None;
    assert!(matches!(
        client.synonyms(&disease).await,
        Err(Error::UnsupportedSynonyms { .. })
    ));
    disease.db_id = Some("C000690".into());
    assert!(matches!(
        client.synonyms(&disease).await,
        Err(Error::InvalidId { .. })
    ));
    assert!(requests.try_recv().is_err());
    disease.db_id = Some("D000690".into());
    client.synonyms(&disease).await.unwrap();
    requests.recv().await.unwrap();
    task.await.unwrap();
}

#[tokio::test]
async fn mesh_errors_and_empty_term_list() {
    let empty = r#"{"descriptor":"http://id.nlm.nih.gov/mesh/D000690","terms":[]}"#;
    let (client, mut requests, task) = server(vec![
        (429, "Retry-After: 5\r\n", "slow down"),
        (200, "", "not JSON"),
        (200, "", empty),
    ])
    .await;
    let id: MeshDescriptorId = "D000690".parse().unwrap();
    match client.mesh_synonyms(&id).await.unwrap_err() {
        Error::Http {
            status,
            retry_after,
            ..
        } => {
            assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(retry_after.as_deref(), Some("5"));
        }
        error => panic!("unexpected {error}"),
    }
    requests.recv().await.unwrap();
    assert!(matches!(
        client.mesh_synonyms(&id).await,
        Err(Error::Json(_))
    ));
    requests.recv().await.unwrap();
    assert!(client.mesh_synonyms(&id).await.unwrap().terms.is_empty());
    requests.recv().await.unwrap();
    task.await.unwrap();
}
