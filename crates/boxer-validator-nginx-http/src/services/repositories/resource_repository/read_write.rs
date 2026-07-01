use crate::http::controllers::v1::resource_set::models::SchemaBoundResourceSetRegistration;
use crate::services::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;

pub type ResourceDiscoveryDocumentRepository = dyn UpsertRepositoryWithDelete<
    (String, String),
    SchemaBoundResourceSetRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

impl UpsertRepositoryWithDelete<(String, String), SchemaBoundResourceSetRegistration>
    for KubernetesRepository<ResourceDiscoveryDocument>
{
}
