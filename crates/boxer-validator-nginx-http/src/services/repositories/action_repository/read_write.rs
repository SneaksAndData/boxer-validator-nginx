use crate::http::controllers::v1::action_set::models::SchemaBoundActionSetRegistration;
use crate::services::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;

pub type ActionDataRepository = dyn UpsertRepositoryWithDelete<
    (String, String),
    SchemaBoundActionSetRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

impl UpsertRepositoryWithDelete<(String, String), SchemaBoundActionSetRegistration>
    for KubernetesRepository<ActionDiscoveryDocument>
{
}
