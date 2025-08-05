use crate::services::repositories::resource_repository::ResourceReadOnlyRepository;
use std::sync::Arc;

pub trait ResourceRepositorySource: Send + Sync {
    fn get_resource_repository(&self) -> Arc<ResourceReadOnlyRepository>;
}
