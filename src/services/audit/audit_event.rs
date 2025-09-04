use cedar_policy::{Decision, Diagnostics, EntityUid, Response};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct AccessAuditEvent {
    action: String,
    actor: String,
    resource: String,
    decision: Decision,
    reason: Reason,
}

// impl ToValue for AccessAuditEvent {
//     fn to_value(&self) -> log::kv::Value {
//         let mut v = log::kv::Value::from(());
//         v.
//         let mut map = std::collections::HashMap::new();
//         map.insert("action", self.action.to_value());
//         map.insert("actor", self.actor.to_value());
//         map.insert("resource", self.resource.to_value());
//         map.insert("decision", format!("{:?}", self.decision).to_value());
//         map.insert("policies", format!("{:?}", self.reason.policies).to_value());
//         map.insert("errors", format!("{:?}", self.reason.errors).to_value());
//         map.into()
//     }
// }

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
