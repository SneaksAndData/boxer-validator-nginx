use crate::models::request_context::RequestContext;
use crate::services::action_repository;
use crate::services::action_repository::models::HTTPMethod::Get;
use crate::services::action_repository::models::RequestSegment;
use crate::services::action_repository::models::RequestSegment::{Hostname, Parameter, Static, Verb};
use crate::services::action_repository::{ActionData, TrieData};
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;
use rstest::rstest;
use std::cmp::Ordering;
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

#[rstest]
fn test_reflexivity(
    #[values(Verb(Get), Hostname("example.com".to_string()), Static("api".to_string()), Parameter)] x: RequestSegment,
) {
    assert_eq!(x.cmp(&x), Ordering::Equal);
}

#[rstest]
fn test_symmetry(
    #[values(Verb(Get), Hostname("example.com".to_string()), Static("api".to_string()), Parameter)] x: RequestSegment,
    #[values(Verb(Get), Hostname("example.com".to_string()), Static("api".to_string()), Parameter)] y: RequestSegment,
) {
    let ordering = x.cmp(&y);

    match ordering {
        Ordering::Less => assert_eq!(y.cmp(&x), Ordering::Greater),
        Ordering::Equal => assert_eq!(y.cmp(&x), Ordering::Equal),
        Ordering::Greater => assert_eq!(y.cmp(&x), Ordering::Less),
    }
}

#[rstest]
fn test_transitivity(
    #[values(Verb(Get), Hostname("example.com".to_string()), Static("api".to_string()), Parameter)] x: RequestSegment,
    #[values(Verb(Get), Hostname("example.com".to_string()), Static("api".to_string()), Parameter)] y: RequestSegment,
    #[values(Verb(Get), Hostname("example.com".to_string()), Static("api".to_string()), Parameter)] z: RequestSegment,
) {
    let ordering = x.cmp(&y);

    match ordering {
        // if x < y and y < z, then x < z
        // if x < y and y == z, then x < z
        Ordering::Less => match y.cmp(&z) {
            Ordering::Less => assert_eq!(x.cmp(&z), Ordering::Less),
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Less),
            _ => assert_eq!(true, true), // Skip this case if y > z, we cannot guarantee x < z
        },

        // if x == y and y == z, then x == z
        Ordering::Equal => match y.cmp(&z) {
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Equal),
            _ => assert_eq!(true, x.cmp(&z) == Ordering::Greater || x.cmp(&z) == Ordering::Less),
        },

        // if x > y and y > z, then x > z
        // if x > y and y == z, then x > z
        Ordering::Greater => match y.cmp(&z) {
            Ordering::Greater => assert_eq!(x.cmp(&z), Ordering::Greater),
            Ordering::Equal => assert_eq!(x.cmp(&z), Ordering::Greater),
            _ => assert_eq!(true, true), // Skip this case if y < z, we cannot guarantee x > z
        },
    }
}
