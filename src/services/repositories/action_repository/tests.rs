use super::*;
use crate::http::controllers::action_set::models::{ActionRouteRegistration, ActionSetRegistration};
use crate::models::request_context::RequestContext;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::{
    KubernetesResourceManagerConfig, ListenerConfig,
};
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
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
    listener_config: ListenerConfig,
    lookup_trie: Arc<TrieRepositoryData<RequestSegment>>,
    lookup: ReadOnlyRepositoryBackend<TrieRepositoryData<RequestSegment>, ActionDiscoveryDocument>,
}

impl AsyncTestContext for KubernetesActionRepositoryTest {
    async fn setup() -> Self {
        let parent = SpinLockKubernetesResourceManagerTestContext::setup().await;
        let listener_config = parent.config.listener_config.clone();
        let config = KubernetesResourceManagerConfig {
            namespace: parent.config.namespace.clone(),
            kubeconfig: parent.config.kubeconfig.clone(),
            field_manager: "unit-test".to_string(),
            listener_config: listener_config.clone(),
        };
        let lookup_trie = Arc::new(TrieRepositoryData::<RequestSegment>::new());
        let lookup = ReadOnlyRepositoryBackend::start(config, lookup_trie.clone())
            .await
            .unwrap();

        let repository = Arc::new(KubernetesRepository {
            resource_manager: parent.manager,
            operation_timeout: parent.config.listener_config.operation_timeout,
        });
        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
            listener_config,
            lookup,
            lookup_trie,
        }
    }
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_create_schema(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange
    let name = "action-discovery-document";
    let registration = ActionSetRegistration {
        hostname: "www.example.com".to_string(),
        routes: vec![ActionRouteRegistration {
            method: "GET".to_string(),
            route_template: "api/v1/resources".to_string(),
            action_uid: "PhotoApp::Photo::\"vacationPhoto.jpg\"".to_string(),
        }],
    };

    ctx.repository
        .upsert(name.to_string(), registration)
        .await
        .expect("Failed to upsert schema");

    // Act
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources".to_string(),
        "GET".to_string(),
    );
    let key = request_context.try_into().unwrap();

    let after = ctx.lookup_trie.get(key).await;

    // Assert
    assert!(after.is_ok());
}
