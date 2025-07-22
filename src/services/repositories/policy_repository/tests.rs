mod test_data;

use crate::services::repositories::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::policy_repository::models::PolicyResource;
use crate::services::repositories::policy_repository::tests::test_data::{test_policy, test_updated_policy};
use crate::services::repositories::policy_repository::{PolicyRepository, PolicyRepositoryData};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::testing::{create_namespace, get_kubeconfig};
use collection_macros::btreemap;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{Api, Client};
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};

struct PolicyRepositoryTestContext {
    raw_api: Arc<Api<ConfigMap>>,
    repository: Arc<ReadOnlyRepositoryBackend>,
    policy_repository: Arc<PolicyRepository>,
    namespace: String,
}

impl AsyncTestContext for PolicyRepositoryTestContext {
    async fn setup() -> PolicyRepositoryTestContext {
        let namespace = create_namespace().await.expect("Failed to create namespace");
        let config = get_kubeconfig().await.expect("Failed to create config");
        let client = Client::try_from(config.clone()).expect("Failed to create client");

        let raw_api: Api<ConfigMap> = Api::namespaced(client.clone(), &namespace);

        let config = KubernetesResourceManagerConfig {
            namespace: namespace.clone(),
            label_selector_key: "repository.boxer.io/type".to_string(),
            label_selector_value: "policy".to_string(),
            lease_name: "actions".to_string(),
            kubeconfig: config,
            lease_duration: Duration::from_secs(5),
            renew_deadline: Duration::from_secs(3),
            claimant: "boxer".to_string(),
        };

        let policy_repository = Arc::new(PolicyRepositoryData::new());
        let repository = ReadOnlyRepositoryBackend::start(config, policy_repository.clone())
            .await
            .expect("Failed to start ActionReadOnlyRepository");

        PolicyRepositoryTestContext {
            raw_api: Arc::new(raw_api),
            repository: Arc::new(repository),
            policy_repository,
            namespace,
        }
    }

    async fn teardown(self) {
        <ReadOnlyRepositoryBackend as KubernetesResourceWatcher<PolicyResource>>::stop(&self.repository).unwrap()
    }
}

fn test_object_meta(name: String, namespace: String) -> ObjectMeta {
    ObjectMeta {
        name: Some(name),
        namespace: Some(namespace),
        labels: Some(btreemap! {
            "repository.boxer.io/type".to_string() => "policy".to_string(),
        }),
        ..Default::default()
    }
}

#[test_context(PolicyRepositoryTestContext)]
#[tokio::test]
async fn test_reading_existing_policies(ctx: &mut PolicyRepositoryTestContext) {
    let name = "test-policies";
    let document = test_policy();
    let cm = ConfigMap {
        metadata: test_object_meta(name.to_string(), ctx.namespace.to_string()),
        data: Some(btreemap! {
            "policies".to_string() => document,
        }),
        ..Default::default()
    };
    ctx.raw_api
        .create(&Default::default(), &cm)
        .await
        .expect("Unable to create resource");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let policy = ctx.policy_repository.get(()).await.expect("Failed to get policy");

    assert_eq!(policy.to_cedar().unwrap(), test_policy());
}

#[test_context(PolicyRepositoryTestContext)]
#[tokio::test]
async fn test_update_policies(ctx: &mut PolicyRepositoryTestContext) {
    let name = "test-policies";
    let document = test_policy();
    let cm = ConfigMap {
        metadata: test_object_meta(name.to_string(), ctx.namespace.to_string()),
        data: Some(btreemap! {
            "policies".to_string() => document,
        }),
        ..Default::default()
    };
    ctx.raw_api
        .create(&Default::default(), &cm)
        .await
        .expect("Unable to create resource");

    tokio::time::sleep(Duration::from_secs(2)).await;
    let document = test_updated_policy();
    let cm = ConfigMap {
        metadata: test_object_meta(name.to_string(), ctx.namespace.to_string()),
        data: Some(btreemap! {
            "policies".to_string() => document,
        }),
        ..Default::default()
    };
    ctx.raw_api
        .replace(name, &Default::default(), &cm)
        .await
        .expect("Unable to create resource");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let policy = ctx.policy_repository.get(()).await.expect("Failed to get policy");

    assert_eq!(policy.to_cedar().unwrap(), test_updated_policy());
}
