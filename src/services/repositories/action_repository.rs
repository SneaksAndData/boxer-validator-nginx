pub mod models;
pub mod read_only;
pub mod read_write;
#[cfg(test)]
mod tests;

use crate::http::controllers::action_set::models::ActionSetRegistration;
use crate::services::repositories::action_repository::models::ActionDiscoveryDocument;
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
    ReadOnlyRepository<String, ActionSetRegistration, ReadError = anyhow::Error>
    + UpsertRepository<String, ActionSetRegistration, Error = anyhow::Error>
    + CanDelete<String, ActionSetRegistration, DeleteError = anyhow::Error>
{
}

pub type ActionReadOnlyRepository = dyn ActionReadOnlyRepositoryInterface + Send + Sync;

pub type ActionRepository = dyn ActionRepositoryInterface + Send + Sync;
