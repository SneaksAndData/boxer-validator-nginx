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
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            action = event.action.as_str(),
            actor = event.actor.as_str(),
            resource = event.resource.as_str(),
            decision:serde = event.decision,
            reason_policies:serde = event.reason.policies,
            reason_errors:serde = event.reason.errors;

            // The log message
            "Audit Event access to resource: {:?}", event.resource());

        Ok(())
    }
}
