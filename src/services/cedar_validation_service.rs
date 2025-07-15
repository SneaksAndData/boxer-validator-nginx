use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::models::request_context::RequestContext;
use crate::services::action_repository::{ActionReadOnlyRepository, ActionRepository};
use crate::services::base::schema_provider::SchemaProvider;
use crate::services::base::validation_service::ValidationService;
use async_trait::async_trait;
use cedar_policy::{Authorizer, Context, Entities, EntityId, EntityTypeName, EntityUid, Request};
use log::{debug, info};
use std::str::FromStr;
use std::sync::Arc;

pub struct CedarValidationService {
    authorizer: Authorizer,
    #[allow(dead_code)]
    schema_provider: Arc<dyn SchemaProvider>,
    action_repository: Arc<ActionReadOnlyRepository>,
}

impl CedarValidationService {
    pub fn new(schema_provider: Arc<dyn SchemaProvider>, action_repository: Arc<ActionReadOnlyRepository>) -> Self {
        CedarValidationService {
            authorizer: Authorizer::new(),
            schema_provider,
            action_repository,
        }
    }
}

#[async_trait]
impl ValidationService for CedarValidationService {
    async fn validate(&self, boxer_claims: BoxerClaims, request_context: RequestContext) -> Result<(), anyhow::Error> {
        let schema = self.schema_provider.get_schema(&boxer_claims).await?;
        debug!("Cedar validation schemas: {:?}", schema);

        let policy_set = boxer_claims.parse()?;
        let actor: EntityUid = boxer_claims.try_into()?;
        let action = self.action_repository.get(request_context.clone().try_into()?).await?;

        let resource = request_context.to_resource()?;

        let entities = Entities::empty();
        let request = Request::new(actor, action, resource, Context::empty(), None)?;
        let answer = self.authorizer.is_authorized(&request, &policy_set, &entities);

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
