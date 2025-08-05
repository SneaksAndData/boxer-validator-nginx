use crate::services::repositories::policy_repository::PolicyReadOnlyRepository;
use std::sync::Arc;

pub trait PolicyRepositorySource: Send + Sync {
    fn get_policy_readonly_repository(&self) -> Arc<PolicyReadOnlyRepository>;
}
