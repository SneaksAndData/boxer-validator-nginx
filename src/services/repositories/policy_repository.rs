pub mod models;

pub mod read_only;
pub mod read_write;

use crate::services::repositories::policy_repository::models::PolicyDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use cedar_policy::PolicySet;

pub trait PolicyReadOnlyRepositoryInterface:
    ReadOnlyRepository<(), PolicySet, ReadError = anyhow::Error> + ResourceUpdateHandler<PolicyDocument>
{
}

pub trait PolicyRepositoryInterface:
    ReadOnlyRepository<String, PolicySet, ReadError = anyhow::Error>
    + UpsertRepository<String, PolicySet, Error = anyhow::Error>
    + CanDelete<String, PolicySet, DeleteError = anyhow::Error>
{
}

pub type PolicyReadOnlyRepository = dyn PolicyReadOnlyRepositoryInterface + Send + Sync;

pub type PolicyRepository = dyn PolicyRepositoryInterface + Send + Sync;
