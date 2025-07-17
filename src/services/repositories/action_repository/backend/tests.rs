use crate::models::request_context::RequestContext;
use crate::services::repositories::action_repository::backend::test_data::{test_routes, test_updated_routes};
use crate::services::repositories::action_repository::backend::ActionRepositoryBackend;
use crate::services::repositories::action_repository::ActionData;
use crate::services::repositories::models::RequestSegment;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use boxer_core::testing::{create_namespace, get_kubeconfig};
use collection_macros::btreemap;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{Api, Client};
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};

struct ActionRepositoryTestContext {
    raw_api: Arc<Api<ConfigMap>>,
    repository: Arc<ActionRepositoryBackend>,
    action_data: Arc<ActionData>,
    namespace: String,
}

impl AsyncTestContext for ActionRepositoryTestContext {
    async fn setup() -> ActionRepositoryTestContext {
        let namespace = create_namespace().await.expect("Failed to create namespace");
        let config = get_kubeconfig().await.expect("Failed to create config");
        let client = Client::try_from(config.clone()).expect("Failed to create client");

        let raw_api: Api<ConfigMap> = Api::namespaced(client.clone(), &namespace);

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

        let action_data = ActionData::new();
        let repository = ActionRepositoryBackend::start(config, action_data.clone())
            .await
            .expect("Failed to start ActionReadOnlyRepository");

        ActionRepositoryTestContext {
            raw_api: Arc::new(raw_api),
            repository: Arc::new(repository),
            action_data,
            namespace,
        }
    }

    async fn teardown(self) {
        self.repository.stop().unwrap()
    }
}

fn test_object_meta(name: String, namespace: String) -> ObjectMeta {
    ObjectMeta {
        name: Some(name),
        namespace: Some(namespace),
        labels: Some(btreemap! {
            "repository.boxer.io/type".to_string() => "action".to_string(),
        }),
        ..Default::default()
    }
}

#[test_context(ActionRepositoryTestContext)]
#[tokio::test]
async fn test_reading_existing_actions(ctx: &mut ActionRepositoryTestContext) {
    let name = "test-actions";
    let document = test_routes();
    let cm = ConfigMap {
        metadata: test_object_meta(name.to_string(), ctx.namespace.to_string()),
        data: Some(btreemap! {
            "actions".to_string() => serde_json::to_string_pretty(&document).expect("Failed to serialize document"),
        }),
        ..Default::default()
    };
    ctx.raw_api
        .create(&Default::default(), &cm)
        .await
        .expect("Unable to create resource");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let request_context = RequestContext::new("https://api.example.com/resources/1234/".to_string(), "GET".to_string());
    let key: Vec<RequestSegment> = Vec::try_from(request_context.clone()).expect("Failed to parse key");

    let action = ctx.action_data.get(key).await.expect("Failed to get action");

    assert_eq!(action.to_string(), "TestApp::\"ReadResource\"");
}

#[test_context(ActionRepositoryTestContext)]
#[tokio::test]
async fn test_updates(ctx: &mut ActionRepositoryTestContext) {
    let name = "test-actions";
    let cm = ConfigMap {
        metadata: test_object_meta(name.to_string(), ctx.namespace.to_string()),
        data: Some(btreemap! {
            "actions".to_string() => serde_json::to_string_pretty(&test_routes()).expect("Failed to serialize document"),
        }),
        ..Default::default()
    };
    ctx.raw_api
        .create(&Default::default(), &cm)
        .await
        .expect("Unable to create resource");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let request_context = RequestContext::new(
        "https://api.v2.example.com/api/v1/items/1234/watchers/efsgsze6/watch".to_string(),
        "POST".to_string(),
    );
    let key: Vec<RequestSegment> = Vec::try_from(request_context.clone()).expect("Failed to parse key");

    let action = ctx.action_data.get(key).await;
    assert_eq!(action.is_err(), true);

    let cm = ConfigMap {
        metadata: kube::api::ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(ctx.namespace.clone()),
            labels: Some(btreemap! {
                "repository.boxer.io/type".to_string() => "action".to_string(),
            }),
            ..Default::default()
        },
        data: Some(btreemap! {
            "actions".to_string() => serde_json::to_string_pretty(&test_updated_routes()).expect("Failed to serialize document"),
        }),
        ..Default::default()
    };
    ctx.raw_api
        .replace("test-actions", &Default::default(), &cm)
        .await
        .expect("Unable to create resource");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let key: Vec<RequestSegment> = Vec::try_from(request_context.clone()).expect("Failed to parse key");

    let action = ctx.action_data.get(key).await;
    assert_eq!(action.is_err(), false);
    assert_eq!(action.unwrap().to_string(), "TestApp::\"WatchResource\"");
}
