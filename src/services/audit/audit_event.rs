use cedar_policy::{Decision, Diagnostics, EntityUid, Response};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Reason {
    #[allow(dead_code)]
    pub policies: HashSet<String>,
    #[allow(dead_code)]
    pub errors: HashSet<String>,
}

impl From<&Diagnostics> for Reason {
    fn from(diagnostics: &Diagnostics) -> Self {
        Self {
            policies: diagnostics.reason().map(|p| p.to_string().clone()).collect(),
            errors: diagnostics.errors().map(|e| e.to_string().clone()).collect(),
        }
    }
}

#[derive(Debug)]
pub struct AccessAuditEvent {
    #[allow(dead_code)]
    action: String,
    #[allow(dead_code)]
    actor: String,
    #[allow(dead_code)]
    resource: String,
    #[allow(dead_code)]
    decision: Decision,
    #[allow(dead_code)]
    reason: Reason,
}

impl AccessAuditEvent {
    pub fn new(actor: &EntityUid, action: &EntityUid, resource: &EntityUid, response: &Response) -> Self {
        Self {
            action: action.to_string(),
            actor: actor.to_string(),
            resource: resource.to_string(),
            decision: response.decision(),
            reason: Reason::from(response.diagnostics()),
        }
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }
}
