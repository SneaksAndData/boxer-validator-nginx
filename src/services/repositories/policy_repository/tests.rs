use crate::http::controllers::v1::policy_set::models::SchemaBoundPolicySetRegistration;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::policy_repository;
use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::repositories::policy_repository::read_only::PolicyRepositoryData;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcherRunner;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::service_provider::ServiceProvider;
use boxer_core::testing::api_extensions::WaitForResource;
use boxer_core::testing::spin_lock_kubernetes_resource_manager_context::SpinLockKubernetesResourceManagerTestContext;
use cedar_policy::PolicySet;
use kube::Api;
use log::LevelFilter;
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);
struct KubernetesSchemaRepositoryTest {
    readonly_repository: Arc<ReadOnlyRepositoryBackend<PolicyRepositoryData, PolicyDocument, String, PolicySet>>,
    readwrite_repository: Arc<PolicyDataRepository>,
    api: Api<PolicyDocument>,
    namespace: String,
}
impl AsyncTestContext for KubernetesSchemaRepositoryTest {
    async fn setup() -> KubernetesSchemaRepositoryTest {
        let parent = SpinLockKubernetesResourceManagerTestContext::setup().await;

        let lookup_trie = policy_repository::read_only::new();
        let mut readonly_repository = ReadOnlyRepositoryBackend::new(lookup_trie.clone(), lookup_trie);
        readonly_repository.start(parent.config.clone()).await.unwrap();
        let readonly_repository = Arc::new(readonly_repository);

        let readwrite_repository = Arc::new(KubernetesRepository {
            resource_manager: parent.manager,
            operation_timeout: parent.config.operation_timeout,
        });

        Self {
            readonly_repository,
            readwrite_repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
        }
    }
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_create_policy(ctx: &mut KubernetesSchemaRepositoryTest) {
    env_logger::builder().filter_level(LevelFilter::Debug).init();

    let policy_str = r#"permit(
    principal == User::"alice",
    action == Action::"read",
    resource == Document::"secret"
);"#;

    let name = "test-policy";
    let schema = "test-schema";
    let reg = SchemaBoundPolicySetRegistration {
        schema: schema.to_string(),
        policy: policy_str.to_string(),
    };

    ctx.readwrite_repository
        .upsert((schema.to_string(), name.to_string()), reg)
        .await
        .expect("Failed to upsert policy");

    ctx.api
        .wait_for_creation(
            "test-schema-test-policy".to_string(),
            ctx.namespace.to_string(),
            DEFAULT_TEST_TIMEOUT,
        )
        .await;

    let policy = ctx.readonly_repository.get().get(schema.to_string()).await.unwrap();

    assert_eq!(policy.to_cedar().unwrap(), policy_str.to_string());
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_create_multiple_policies(ctx: &mut KubernetesSchemaRepositoryTest) {
    env_logger::builder().filter_level(LevelFilter::Debug).init();

    let policy_1 = r#"permit(
    principal == User::"alice",
    action == Action::"read",
    resource == Document::"secret"
);"#;

    let policy_2 = r#"permit(
    principal == User::"alice",
    action == Action::"read",
    resource == Document::"secret"
);"#;

    let name = "test-policy";
    let schema = "test-schema";
    let reg = SchemaBoundPolicySetRegistration {
        schema: schema.to_string(),
        policy: policy_1.to_string(),
    };

    ctx.readwrite_repository
        .upsert((schema.to_string(), format!("{}-1", name)), reg)
        .await
        .expect("Failed to upsert policy");

    let reg2 = SchemaBoundPolicySetRegistration {
        schema: schema.to_string(),
        policy: policy_2.to_string(),
    };
    ctx.readwrite_repository
        .upsert((schema.to_string(), format!("{}-2", name)), reg2)
        .await
        .expect("Failed to upsert policy");

    ctx.api
        .wait_for_creation(
            "test-schema-test-policy-2".to_string(),
            ctx.namespace.to_string(),
            DEFAULT_TEST_TIMEOUT,
        )
        .await;

    let policy = ctx.readonly_repository.get().get(schema.to_string()).await.unwrap();

    assert_eq!(policy.to_cedar().unwrap(), format!("{}\n\n{}", policy_1, policy_2));
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_modify_policy(ctx: &mut KubernetesSchemaRepositoryTest) {
    env_logger::builder().filter_level(LevelFilter::Debug).init();

    let policy_initial = r#"permit(
    principal == User::"alice",
    action == Action::"read",
    resource == Document::"secret"
);"#;

    let policy_updated = r#"permit(
    principal == User::"bob",
    action == Action::"write",
    resource == Document::"secret"
);"#;

    let name = "test-policy";
    let schema = "test-schema";

    let reg = SchemaBoundPolicySetRegistration {
        schema: schema.to_string(),
        policy: policy_initial.to_string(),
    };

    ctx.readwrite_repository
        .upsert((schema.to_string(), name.to_string()), reg)
        .await
        .expect("Failed to upsert policy");

    ctx.api
        .wait_for_creation(
            "test-schema-test-policy".to_string(),
            ctx.namespace.to_string(),
            DEFAULT_TEST_TIMEOUT,
        )
        .await;

    let policy = ctx.readonly_repository.get().get(schema.to_string()).await.unwrap();

    assert_eq!(policy.to_cedar().unwrap(), policy_initial.to_string());

    let reg = SchemaBoundPolicySetRegistration {
        schema: schema.to_string(),
        policy: policy_updated.to_string(),
    };

    ctx.readwrite_repository
        .upsert((schema.to_string(), name.to_string()), reg)
        .await
        .expect("Failed to upsert policy");

    ctx.api
        .wait_for_creation(
            "test-schema-test-policy".to_string(),
            ctx.namespace.to_string(),
            DEFAULT_TEST_TIMEOUT,
        )
        .await;

    let policy = ctx.readonly_repository.get().get(schema.to_string()).await.unwrap();

    assert_eq!(policy.to_cedar().unwrap(), policy_updated.to_string());
}
