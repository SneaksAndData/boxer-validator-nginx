use crate::services::repositories::action_repository::{ActionReadOnlyRepository, ActionRepository};
use std::sync::Arc;

pub trait ActionRepositorySource: Send + Sync {
    fn get_readonly_repository(&self) -> Arc<ActionReadOnlyRepository>;
    #[allow(dead_code)]
    fn get_action_data_repository(&self) -> Arc<ActionRepository>;
}
