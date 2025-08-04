pub mod models;
pub mod read_only;
pub mod read_write;
#[cfg(test)]
mod test_data;
#[cfg(test)]
mod tests;

use crate::services::repositories::action_repository::models::{ActionDiscoveryDocument, ActionDiscoveryDocumentSpec};
use crate::services::repositories::models::RequestSegment;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;

pub trait ActionReadOnlyRepositoryInterface:
    ReadOnlyRepository<Vec<RequestSegment>, EntityUid, ReadError = anyhow::Error>
    + ResourceUpdateHandler<ActionDiscoveryDocument>
{
}

pub trait ActionRepositoryInterface:
    ReadOnlyRepository<String, ActionDiscoveryDocumentSpec, ReadError = anyhow::Error>
    + UpsertRepository<String, ActionDiscoveryDocumentSpec, Error = anyhow::Error>
    + CanDelete<String, ActionDiscoveryDocumentSpec, DeleteError = anyhow::Error>
{
}

pub type ActionReadOnlyRepository = dyn ActionReadOnlyRepositoryInterface + Send + Sync;

pub type ActionRepository = dyn ActionRepositoryInterface + Send + Sync;
