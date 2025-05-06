use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::models::request_context::RequestContext;
use crate::services::base::validation_service::ValidationService;
use cedar_policy::{Authorizer, Context, Entities, EntityId, EntityTypeName, EntityUid, Request};
use log::info;
use std::str::FromStr;

pub struct CedarValidationService {
    authorizer: Authorizer,
}

impl CedarValidationService {
    pub fn new() -> Self {
        CedarValidationService {
            authorizer: Authorizer::new(),
        }
    }
}

impl ValidationService for CedarValidationService {
    fn validate(
        &self,
        boxer_claims: BoxerClaims,
        request_context: RequestContext,
    ) -> Result<(), anyhow::Error> {
        let policy_set = boxer_claims.parse()?;
        let actor: EntityUid = boxer_claims.try_into()?;
        let action = request_context.to_action()?;
        let resource = request_context.to_resource()?;

        let entities = Entities::empty();
        let request = Request::new(
            Some(actor),
            Some(action),
            Some(resource),
            Context::empty(),
            None,
        )?;
        let answer = self
            .authorizer
            .is_authorized(&request, &policy_set, &entities);
        info!("validation {:?}", answer.decision());
        match answer.decision() {
            cedar_policy::Decision::Allow => Ok(()),
            cedar_policy::Decision::Deny => anyhow::bail!("Access denied"),
        }
    }
}

impl TryInto<EntityUid> for BoxerClaims {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<EntityUid, Self::Error> {
        let tp = EntityTypeName::from_str(&self.identity_provider)?;
        let n = EntityId::from_str(&self.user_id)?;
        Ok(EntityUid::from_type_name_and_id(tp, n))
    }
}
