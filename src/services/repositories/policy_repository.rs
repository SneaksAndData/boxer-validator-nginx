pub mod policy_document;

pub mod read_only;
pub mod read_write;

use crate::http::controllers::policy_set::models::PolicySetRegistration;
use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use cedar_policy::PolicySet;

pub trait PolicyReadOnlyRepositoryInterface:
    ReadOnlyRepository<(), PolicySet, ReadError = anyhow::Error> + ResourceUpdateHandler<PolicyDocument>
{
}

pub trait PolicyRepositoryInterface:
    ReadOnlyRepository<String, PolicySetRegistration, ReadError = anyhow::Error>
    + UpsertRepository<String, PolicySetRegistration, Error = anyhow::Error>
    + CanDelete<String, PolicySetRegistration, DeleteError = anyhow::Error>
{
}

pub type PolicyReadOnlyRepository = dyn PolicyReadOnlyRepositoryInterface + Send + Sync;

pub type PolicyRepository = dyn PolicyRepositoryInterface + Send + Sync;
