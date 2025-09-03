use cedar_policy::{Decision, Diagnostics, EntityUid, Response};
use std::collections::HashSet;
use std::fmt::Display;

#[derive(Debug)]
pub struct Reason {
    pub policies: HashSet<String>,
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
    action: String,
    actor: String,
    resource: String,
    decision: Decision,
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
