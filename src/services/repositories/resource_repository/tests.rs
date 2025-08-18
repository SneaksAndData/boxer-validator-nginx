use super::*;
use crate::http::controllers::resource_set::models::{ResourceRouteRegistration, ResourceSetRegistration};
use crate::models::request_context::RequestContext;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
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

struct KubernetesResourceRepositoryTest {
    repository: Arc<ResourceDiscoveryDocumentRepository>,
    api: Api<ResourceDiscoveryDocument>,
    namespace: String,
    lookup: ReadOnlyRepositoryBackend<TrieRepositoryData<PathSegment>, ResourceDiscoveryDocument>,
}

impl AsyncTestContext for KubernetesResourceRepositoryTest {
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
        let lookup_trie = Arc::new(TrieRepositoryData::<PathSegment>::new());
        let lookup = ReadOnlyRepositoryBackend::start(config, lookup_trie.clone())
            .await
            .unwrap();

        let repository = Arc::new(KubernetesRepository {
            resource_manager: parent.manager,
            operation_timeout: parent.config.operation_timeout.clone(),
        });
        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
            lookup,
        }
    }
}

#[test_context(KubernetesResourceRepositoryTest)]
#[tokio::test]
async fn test_create_resource_discovery_document(ctx: &mut KubernetesResourceRepositoryTest) {
    // Arrange
    let name = "resource-discovery-document";
    let registration = ResourceSetRegistration {
        hostname: "www.example.com".to_string(),
        routes: vec![ResourceRouteRegistration {
            route_template: "api/v1/resources".to_string(),
            resource_uid: "PhotoApp::Photo::\"vacationPhoto.jpg\"".to_string(),
        }],
    };

    ctx.repository
        .upsert(("schema".to_string(), name.to_string()), registration)
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

    let after = ctx.lookup.get().get(key).await;

    // Assert
    assert!(after.is_ok());
}
