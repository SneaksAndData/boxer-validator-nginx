use crate::services::audit::audit_event::AccessAuditEvent;
use crate::services::audit::AuditService;
use anyhow::Result;

pub struct LogAuditService;

impl LogAuditService {
    pub fn new() -> Self {
        Self {}
    }
}

impl AuditService for LogAuditService {
    fn record(&self, event: AccessAuditEvent) -> Result<()> {
        log::info!(target: "audit",
            summary:serde = event; "Audit Event access to resource: {:?}", event.resource());
        Ok(())
    }
}
