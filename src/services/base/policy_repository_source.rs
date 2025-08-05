use crate::services::repositories::policy_repository::{PolicyReadOnlyRepository, PolicyRepository};
use std::sync::Arc;

pub trait PolicyRepositorySource: Send + Sync {
    fn get_policy_readonly_repository(&self) -> Arc<PolicyReadOnlyRepository>;
    fn get_policy_data_repository(&self) -> Arc<PolicyRepository>;
}
