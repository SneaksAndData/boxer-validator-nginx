use actix_web::http::Uri;
use cedar_policy::{EntityId, EntityTypeName, EntityUid};
use std::str::FromStr;

#[derive(Debug)]
#[allow(dead_code)]
pub struct RequestContext {
    original_url: String,
    original_method: String,
}

impl RequestContext {
    pub fn new(original_url: String, original_method: String) -> RequestContext {
        RequestContext {
            original_url,
            original_method,
        }
    }

    pub fn to_action(&self) -> anyhow::Result<EntityUid> {
        let tp = EntityTypeName::from_str("Action")?;
        let n = EntityId::from_str(&self.original_method)?;
        Ok(EntityUid::from_type_name_and_id(tp, n))
    }

    pub fn to_resource(&self) -> anyhow::Result<EntityUid> {
        let tp = EntityTypeName::from_str("Http")?;
        let uri = self.original_url.parse::<Uri>()?;
        let name = uri.host().unwrap().to_string() + uri.path();
        let n = EntityId::from_str(&name)?;
        Ok(EntityUid::from_type_name_and_id(tp, n))
    }
}
