use crate::http::filters::jwt_filter::DynamicClaimsCollection;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::convert::TryFrom;

// This claim should be always present in the boxer token
const API_VERSION_KEY: &str = "boxer.sneaksanddata.com/api-version";

// Constants related to a particular API version
const PRINCIPAL_KEY: &str = "boxer.sneaksanddata.com/principal";
const USER_ID_KEY: &str = "boxer.sneaksanddata.com/user-id";
const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";
const SCHEMA_KEY: &str = "boxer.sneaksanddata.com/schema";

#[derive(Debug)]
#[allow(dead_code)]
pub struct BoxerClaims {
    pub api_version: String,
    pub schema: String,
    pub principal: String,
}

impl TryFrom<&DynamicClaimsCollection> for BoxerClaims {
    type Error = anyhow::Error;

    fn try_from(c: &DynamicClaimsCollection) -> Result<Self, Self::Error> {
        let api_version = get_claim(c, API_VERSION_KEY).ok_or(anyhow::anyhow!("Missing api version"))?;
        let schema = get_claim(c, SCHEMA_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;
        let principal = get_claim(c, PRINCIPAL_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;

        Ok(BoxerClaims {
            api_version: api_version.to_string(),
            schema: String::from_utf8(STANDARD.decode(&schema)?)?,
            principal: String::from_utf8(STANDARD.decode(&principal)?)?,
        })
    }
}

fn get_claim(claims: &DynamicClaimsCollection, key: &str) -> Option<String> {
    let value = claims.get(key)?;
    let value = value.as_str()?;
    Some(value.to_owned())
}
