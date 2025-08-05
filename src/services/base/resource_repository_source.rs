use crate::services::repositories::resource_repository::{ResourceReadOnlyRepository, ResourceRepository};
use std::sync::Arc;

pub trait ResourceRepositorySource: Send + Sync {
    fn get_resource_repository(&self) -> Arc<ResourceRepository>;
    fn get_resource_read_only_repository(&self) -> Arc<ResourceReadOnlyRepository>;
}
