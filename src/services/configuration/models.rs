use crate::services::configuration::signature_settings::SignatureSettings;
use anyhow::Result;
use boxer_core::services::observability::open_telemetry::settings::OpenTelemetrySettings;
use duration_string::DurationString;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,
    pub resource_owner_label: String,
    pub operation_timeout: DurationString,
}

#[derive(Debug, Deserialize)]
pub struct BackendSettings {
    pub kubernetes: KubernetesBackendSettings,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub listen_address: SocketAddr,
    pub instance_name: String,
    pub backend: BackendSettings,
    pub opentelemetry: OpenTelemetrySettings,

    // We use JSON-encoded string for signature settings since the validator must support multiple
    // signatures for seamless key rotation. Unfortunately, the config-rs crate does not support
    // deserializing directly into a Vec or HashMap from environment variables.
    pub signatures: String,
}

impl AppSettings {
    pub fn get_signatures(&self) -> Result<SignatureSettings> {
        serde_json::from_str::<SignatureSettings>(self.signatures.as_ref()).map_err(|e| anyhow::anyhow!(e))
    }
}
