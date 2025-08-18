use super::*;
use crate::http::controllers::action_set::models::{ActionRouteRegistration, ActionSetRegistration};
use crate::models::request_context::RequestContext;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::service_provider::ServiceProvider;
use boxer_core::testing::api_extensions::WaitForResource;
use boxer_core::testing::spin_lock_kubernetes_resource_manager_context::SpinLockKubernetesResourceManagerTestContext;
use kube::Api;
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct KubernetesActionRepositoryTest {
    repository: Arc<ActionDataRepository>,
    api: Api<ActionDiscoveryDocument>,
    namespace: String,
    lookup: ReadOnlyRepositoryBackend<TrieRepositoryData<RequestSegment>, ActionDiscoveryDocument>,
}

impl AsyncTestContext for KubernetesActionRepositoryTest {
    async fn setup() -> Self {
        let parent = SpinLockKubernetesResourceManagerTestContext::setup().await;
        let owner_mark = parent.config.owner_mark.clone();
        let operation_timeout = parent.config.operation_timeout.clone();
        let config = KubernetesResourceManagerConfig {
            namespace: parent.config.namespace.clone(),
            kubeconfig: parent.config.kubeconfig.clone(),
            owner_mark,
            operation_timeout: operation_timeout.clone(),
        };
        let lookup_trie = Arc::new(TrieRepositoryData::<RequestSegment>::new());
        let lookup = ReadOnlyRepositoryBackend::start(config, lookup_trie.clone())
            .await
            .unwrap();

        let repository = Arc::new(KubernetesRepository {
            resource_manager: parent.manager,
            operation_timeout: parent.config.operation_timeout,
        });
        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
            lookup,
        }
    }
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_create_action_document(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange

    insert_schema_document(
        ctx,
        "action-discovery-document",
        "api/v1/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;

    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources".to_string(),
        "GET".to_string(),
    );
    let key = request_context.try_into().unwrap();

    let lookup_trie = ctx.lookup.get();
    let result = lookup_trie.get(key).await;

    // Assert
    assert!(result.is_ok());
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_multiple_actions(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange
    insert_schema_document(
        ctx,
        "action-discovery-document-first",
        "api/v1/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;
    insert_schema_document(
        ctx,
        "action-discovery-document-second",
        "api/v2/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;
    let lookup_trie = ctx.lookup.get();

    // Act
    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources".to_string(),
        "GET".to_string(),
    );
    let key: Vec<RequestSegment> = request_context.try_into().unwrap();
    let first_result = lookup_trie.get(key).await;

    let request_context = RequestContext::new(
        "https://www.example.com/api/v2/resources".to_string(),
        "GET".to_string(),
    );
    let key = request_context.try_into().unwrap();

    let second_result = lookup_trie.get(key).await;

    // Assert
    assert!(first_result.is_ok());
    assert!(second_result.is_ok());
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_remove(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange
    insert_schema_document(
        ctx,
        "action-discovery-document-first",
        "api/v1/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;

    insert_schema_document(
        ctx,
        "action-discovery-document-second",
        "api/v2/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;

    let lookup_trie = ctx.lookup.get();

    ctx.repository
        .delete(("schema".to_string(), "action-discovery-document-first".to_string()))
        .await
        .unwrap();

    // Act
    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources".to_string(),
        "GET".to_string(),
    );
    let key: Vec<RequestSegment> = request_context.try_into().unwrap();
    let first_result = lookup_trie.get(key).await;

    // Assert
    assert!(first_result.is_err());
}

async fn insert_schema_document(ctx: &KubernetesActionRepositoryTest, name: &str, route: &str, action_uid: &str) {
    let registration = ActionSetRegistration {
        hostname: "www.example.com".to_string(),
        routes: vec![ActionRouteRegistration {
            method: "GET".to_string(),
            route_template: route.to_string(),
            action_uid: action_uid.to_string(),
        }],
    };

    ctx.repository
        .upsert(("schema".to_string(), name.to_string()), registration)
        .await
        .expect("Failed to upsert schema");

    let key = format!("{}-{}", "schema", name);
    ctx.api
        .wait_for_creation(key, ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;
}
