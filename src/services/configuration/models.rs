use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub instance_name: String,
}
