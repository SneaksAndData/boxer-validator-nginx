use crate::services::repositories::policy_repository::PolicyRepository;
use std::sync::Arc;

pub trait PolicyRepositorySource: Send + Sync {
    fn get_policy_repository(&self) -> Arc<PolicyRepository>;
}
