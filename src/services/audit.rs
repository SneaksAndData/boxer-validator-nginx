pub mod audit_event;
pub mod log_audit_service;

use crate::services::audit::audit_event::AccessAuditEvent;
use anyhow::Result;

pub trait AuditService: Send + Sync {
    fn record(&self, event: AccessAuditEvent) -> Result<()>;
}
