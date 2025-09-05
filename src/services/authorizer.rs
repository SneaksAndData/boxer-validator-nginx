use crate::services::configuration::signature_settings::SignatureSettings;
use anyhow::Result;
use boxer_core::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use collection_macros::hashset;
use josekit::jwe::Dir;
use josekit::jwt;
use serde_json::Value;
use std::collections::HashSet;

pub struct Authorizer {
    pub keys: SignatureSettings,
    pub valid_issuers: HashSet<String>,
    pub valid_audiences: HashSet<String>,
}

impl Authorizer {
    pub fn new(keys: SignatureSettings) -> Self {
        Authorizer {
            keys,
            valid_issuers: hashset! {"boxer.sneaksanddata.com".to_string()},
            valid_audiences: hashset!["boxer.sneaksanddata.com".to_string()],
        }
    }
}

impl Authorizer {
    pub fn validate(&self, raw: &str) -> Result<DynamicClaimsCollection> {
        let header = jwt::decode_header(&raw).map_err(anyhow::Error::from)?;
        let key_id = header
            .claim("kid")
            .and_then(Value::as_str)
            .ok_or(anyhow::anyhow!("No 'kid' in token header"))?
            .to_string();

        let key = self
            .keys
            .get(&key_id)
            .ok_or_else(|| anyhow::anyhow!("No key found for kid: {}", key_id))
            .map_err(anyhow::Error::from)?;
        let decrypter = Dir.decrypter_from_bytes(key).map_err(anyhow::Error::from)?;
        let (payload, header) = jwt::decode_with_decrypter(&raw, &decrypter)?;

        let audiences = header.audience().ok_or(anyhow::anyhow!("No audience"))?;
        if !audiences.iter().any(|a| self.valid_audiences.contains(*a)) {
            return Err(anyhow::anyhow!("No valid audience"));
        }

        let issuer = header.issuer().ok_or(anyhow::anyhow!("No audience"))?;
        if !self.valid_issuers.contains(issuer) {
            return Err(anyhow::anyhow!("Invalid issuer"));
        }

        Ok(payload)
    }
}
