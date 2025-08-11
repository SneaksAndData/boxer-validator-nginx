pub mod policy_document;

pub mod read_only;
pub mod read_write;

use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::PolicySet;

pub trait PolicyReadOnlyRepositoryInterface:
    ReadOnlyRepository<(), PolicySet, ReadError = anyhow::Error> + ResourceUpdateHandler<PolicyDocument>
{
}

pub type PolicyReadOnlyRepository = dyn PolicyReadOnlyRepositoryInterface + Send + Sync;
