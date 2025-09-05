use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SignatureSettings(HashMap<String, String>);

impl SignatureSettings {
    pub fn get(&self, key_id: &str) -> Option<&String> {
        self.0.get(key_id)
    }
}
