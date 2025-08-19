use crate::http::controllers::policy_set::models::SchemaBoundPolicySetRegistration;
use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;

pub type PolicyDataRepository = dyn UpsertRepositoryWithDelete<
    (String, String),
    SchemaBoundPolicySetRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

impl UpsertRepositoryWithDelete<(String, String), SchemaBoundPolicySetRegistration>
    for KubernetesRepository<PolicyDocument>
{
}
