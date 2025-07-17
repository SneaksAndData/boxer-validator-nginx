use crate::services::repositories::action_repository::ActionRepository;
use std::sync::Arc;

pub trait ActionRepositorySource: Send + Sync {
    fn get_actions_repository(&self) -> Arc<dyn ActionRepository>;
}
