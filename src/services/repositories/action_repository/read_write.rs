use crate::http::controllers::action_set::models::ActionSetRegistration;
use crate::services::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;

pub type ActionDataRepository = dyn UpsertRepositoryWithDelete<
    String,
    ActionSetRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

impl UpsertRepositoryWithDelete<String, ActionSetRegistration> for KubernetesRepository<ActionDiscoveryDocument> {}
