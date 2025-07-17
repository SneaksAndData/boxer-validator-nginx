use crate::models::request_context::RequestContext;
use crate::services::repositories::action_repository;
use crate::services::repositories::action_repository::{ActionData, TrieData};
use crate::services::repositories::models::HTTPMethod::Get;
use crate::services::repositories::models::PathSegment::{Parameter, Static};
use crate::services::repositories::models::RequestSegment;
use crate::services::repositories::models::RequestSegment::{Hostname, Path, Verb};
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::EntityUid;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use trie_rs::map::TrieBuilder;

#[tokio::test]
async fn test_can_read() {
    let rc = RequestContext::new("https://example.com/api/v1/resource".to_string(), "GET".to_string());
    let action_repo = Arc::new(ActionData {
        rw_lock: RwLock::new(TrieData {
            builder: Box::new(TrieBuilder::new()),
            maybe_trie: None,
        }),
    });
    let path: Vec<RequestSegment> = rc.clone().try_into().unwrap();
    let entity = EntityUid::from_str("TestApp::\"RunAction\"").unwrap();
    {
        let mut lock = action_repo.rw_lock.write().await;
        lock.builder.push(path.clone(), entity);
        lock.maybe_trie = Some(Arc::new(lock.builder.clone().build()));
    }

    let action = action_repo.get(path).await.unwrap();
    assert_eq!("TestApp::\"RunAction\"", action.to_string());
}

#[tokio::test]
async fn test_can_insert() {
    let rc = RequestContext::new("https://example.com/api/v1/resource".to_string(), "GET".to_string());

    let action_repo = action_repository::new();
    let path: Vec<RequestSegment> = rc.clone().try_into().unwrap();
    let entity = EntityUid::from_str("TestApp::\"RunAction\"").unwrap();
    action_repo.upsert(path, entity).await.unwrap();

    let action = action_repo.get(rc.try_into().unwrap()).await.unwrap();
    assert_eq!("TestApp::\"RunAction\"", action.to_string());
}

#[tokio::test]
async fn test_parameters() {
    let rc = RequestContext::new("https://example.com/api/v1/resource".to_string(), "GET".to_string());
    // The method `GET`, host `example.com` and route `/api/{version}/resource`
    let parametrized_path = vec![
        Hostname("example.com".to_string()),
        Verb(Get),
        Path(Static("api".to_string())),
        Path(Parameter),
        Path(Static("resource".to_string())),
    ];

    let action_repo = action_repository::new();
    let path: Vec<RequestSegment> = rc.clone().try_into().unwrap();

    let entity = EntityUid::from_str("TestApp::\"RunAction\"").unwrap();
    action_repo.upsert(path, entity).await.unwrap();

    let action = action_repo.get(parametrized_path).await.unwrap();
    assert_eq!("TestApp::\"RunAction\"", action.to_string());
}

#[tokio::test]
async fn test_missing_parameter_in_route() {
    let rc = RequestContext::new("https://example.com/api/v1/resource".to_string(), "GET".to_string());
    // The method `GET`, host `example.com` and route `/api/{version}/resource`
    let parametrized_path = vec![
        Hostname("example.com".to_string()),
        Verb(Get),
        Path(Static("api".to_string())),
        Path(Static("v2".to_string())),
        Path(Static("resource".to_string())),
    ];

    let action_repo = action_repository::new();
    let path: Vec<RequestSegment> = rc.clone().try_into().unwrap();

    let entity = EntityUid::from_str("TestApp::\"RunAction\"").unwrap();
    action_repo.upsert(path, entity).await.unwrap();

    let result = action_repo.get(parametrized_path).await;
    assert_eq!(result.is_err(), true);
}

#[tokio::test]
async fn test_missing_parameter_in_the_end() {
    let rc = RequestContext::new("https://example.com/api/v1/resource".to_string(), "GET".to_string());
    // The method `GET`, host `example.com` and route `/api/{version}/resource`
    let parametrized_path = vec![
        Hostname("example.com".to_string()),
        Verb(Get),
        Path(Static("api".to_string())),
        Path(Static("v1".to_string())),
        Path(Static("resource".to_string())),
        Path(Parameter),
    ];

    let action_repo = action_repository::new();
    let path: Vec<RequestSegment> = rc.clone().try_into().unwrap();

    let entity = EntityUid::from_str("TestApp::\"RunAction\"").unwrap();
    action_repo.upsert(path, entity).await.unwrap();

    let result = action_repo.get(parametrized_path).await;
    assert_eq!(result.is_err(), true);
}
