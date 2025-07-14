use crate::http::filters::jwt_filter::DynamicClaimsCollection;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cedar_policy::PolicySet;
use flate2::read::ZlibDecoder;
use std::convert::TryFrom;
use std::io::Read;

// This claim should be always present in the boxer token
const API_VERSION_KEY: &str = "boxer.sneaksanddata.com/api-version";

// Constants related to a particular API version
const POLICY_KEY: &str = "boxer.sneaksanddata.com/policy";
const USER_ID_KEY: &str = "boxer.sneaksanddata.com/user-id";
const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";
const SCHEMA_KEY: &str = "boxer.sneaksanddata.com/schema";

#[derive(Debug)]
#[allow(dead_code)]
pub struct BoxerClaims {
    pub api_version: String,
    pub policy: String,
    pub user_id: String,
    pub identity_provider: String,
    pub schema: String,
}

impl BoxerClaims {
    pub fn parse(&self) -> Result<PolicySet, anyhow::Error> {
        let policy = self.policy.clone();
        let decompressed_policy = {
            let bytes = STANDARD.decode(policy)?;
            let mut decoder = ZlibDecoder::new(&bytes[..]);
            let mut result = String::new();
            decoder.read_to_string(&mut result)?;
            result
        };
        decompressed_policy.parse::<PolicySet>().map_err(|e| anyhow::anyhow!(e))
    }
}

impl TryFrom<&DynamicClaimsCollection> for BoxerClaims {
    type Error = anyhow::Error;

    fn try_from(c: &DynamicClaimsCollection) -> Result<Self, Self::Error> {
        let api_version = get_claim(c, API_VERSION_KEY).ok_or(anyhow::anyhow!("Missing api version"))?;
        let policy = get_claim(c, POLICY_KEY).ok_or(anyhow::anyhow!("Missing policy"))?;
        let user_id = get_claim(c, USER_ID_KEY).ok_or(anyhow::anyhow!("Missing user id"))?;
        let identity_provider =
            get_claim(c, IDENTITY_PROVIDER_KEY).ok_or(anyhow::anyhow!("Missing identity provider"))?;
        let schema = get_claim(c, SCHEMA_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;

        Ok(BoxerClaims {
            api_version: api_version.to_string(),
            policy: policy.to_string(),
            user_id: user_id.to_string(),
            identity_provider: identity_provider.to_string(),
            schema: String::from_utf8(STANDARD.decode(&schema)?)?,
        })
    }
}

fn get_claim(claims: &DynamicClaimsCollection, key: &str) -> Option<String> {
    let value = claims.get(key)?;
    let value = value.as_str()?;
    Some(value.to_owned())
}
