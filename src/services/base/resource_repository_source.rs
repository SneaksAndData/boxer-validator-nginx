use crate::services::repositories::resource_repository::ResourceRepository;
use std::sync::Arc;

pub trait ResourceRepositorySource: Send + Sync {
    fn get_resource_repository(&self) -> Arc<ResourceRepository>;
}
