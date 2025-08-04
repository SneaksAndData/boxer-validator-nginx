use crate::models::request_context::RequestContext;
use crate::services::repositories::action_repository;
use crate::services::repositories::action_repository::models::{ActionDiscoveryDocumentSpec, ActionRoute};
use crate::services::repositories::action_repository::read_only::ActionLookupTrie;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::models::HTTPMethod::Get;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use boxer_core::testing::{create_namespace, get_kubeconfig};
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};

struct ActionRepositoryTestContext {
    #[allow(dead_code)]
    lookup_listener: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    repository_listener: Arc<ReadOnlyRepositoryBackend>,
    repository: Arc<ActionDataRepository>,
    lookup: Arc<ActionLookupTrie>,
}

impl AsyncTestContext for ActionRepositoryTestContext {
    async fn setup() -> ActionRepositoryTestContext {
        let namespace = create_namespace().await.expect("Failed to create namespace");
        let config = get_kubeconfig().await.expect("Failed to create config");

        let config = KubernetesResourceManagerConfig {
            namespace: namespace.clone(),
            label_selector_key: "repository.boxer.io/type".to_string(),
            label_selector_value: "action".to_string(),
            lease_name: "actions".to_string(),
            kubeconfig: config,
            lease_duration: Duration::from_secs(5),
            renew_deadline: Duration::from_secs(3),
            claimant: "boxer".to_string(),
        };

        let lookup = action_repository::read_only::new();
        let lookup_listener = Arc::new(
            ReadOnlyRepositoryBackend::start(config.clone(), lookup.clone())
                .await
                .expect("Failed to start ActionReadOnlyRepository"),
        );

        let repository = action_repository::read_write::new(config.clone()).await;
        let repository_listener = Arc::new(
            ReadOnlyRepositoryBackend::start(config, lookup.clone())
                .await
                .expect("Failed to start ActionReadOnlyRepository"),
        );

        ActionRepositoryTestContext {
            lookup_listener,
            repository_listener,
            repository,
            lookup,
        }
    }

    async fn teardown(self) {
        // self.lookup_listener.stop().unwrap();
        // self.repository_listener.stop().unwrap();
    }
}

#[test_context(ActionRepositoryTestContext)]
#[tokio::test]
async fn test_create_read_document(ctx: &mut ActionRepositoryTestContext) {
    let spec = ActionDiscoveryDocumentSpec {
        active: true,
        hostname: "www.example.com".to_string(),
        routes: vec![ActionRoute {
            action_uid: "TestApp::\"RunAction\"".to_string(),
            route_template: "/api/v1/resource".to_string(),
            method: Get,
        }],
    };

    ctx.repository
        .upsert("action-discovery-document".to_string(), spec.clone())
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;
    let document = ctx
        .repository
        .get("action-discovery-document".to_string())
        .await
        .unwrap();

    assert_eq!(document.hostname, "www.example.com");
}

#[test_context(ActionRepositoryTestContext)]
#[tokio::test]
async fn test_create_read_path(ctx: &mut ActionRepositoryTestContext) {
    let route_template = "/api/v1/resource".to_string();
    let action_uid = "TestApp::\"RunAction\"".to_string();

    let spec = ActionDiscoveryDocumentSpec {
        active: true,
        hostname: "www.example.com".to_string(),
        routes: vec![ActionRoute {
            action_uid: action_uid.clone(),
            route_template: route_template.clone(),
            method: Get,
        }],
    };

    ctx.repository
        .upsert("action-discovery-document".to_string(), spec.clone())
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    let rc = RequestContext {
        original_method: "GET".to_string(),
        original_url: "https://www.example.com/api/v1/resource".to_string(),
    };
    let action = ctx.lookup.get(rc.try_into().unwrap()).await.unwrap();

    assert_eq!(action.to_string(), action_uid);
}

#[test_context(ActionRepositoryTestContext)]
#[tokio::test]
async fn test_reacts_on_deletions(ctx: &mut ActionRepositoryTestContext) {
    let route_template = "/api/v1/resource".to_string();
    let action_uid = "TestApp::\"RunAction\"".to_string();

    let spec = ActionDiscoveryDocumentSpec {
        active: true,
        hostname: "www.example.com".to_string(),
        routes: vec![ActionRoute {
            action_uid: action_uid.clone(),
            route_template: route_template.clone(),
            method: Get,
        }],
    };

    ctx.repository
        .upsert("action-discovery-document-deleted".to_string(), spec.clone())
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    let rc = RequestContext {
        original_method: "GET".to_string(),
        original_url: "https://www.example.com/api/v1/resource".to_string(),
    };
    let action = ctx.lookup.get(rc.clone().try_into().unwrap()).await.unwrap();
    assert_eq!(action.to_string(), action_uid);

    ctx.repository
        .delete("action-discovery-document-deleted".to_string())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;

    let action = ctx.lookup.get(rc.try_into().unwrap()).await;
    assert!(action.is_err(), "Action should not be found after deletion");
}
