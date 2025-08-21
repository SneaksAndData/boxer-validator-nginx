use crate::models::request_context::RequestContext;
use crate::services::base::schema_provider::SchemaProvider;
use crate::services::base::validation_service::ValidationService;
use crate::services::repositories::action_repository::ActionReadOnlyRepository;
use crate::services::repositories::policy_repository::PolicyReadOnlyRepository;
use crate::services::repositories::resource_repository::ResourceReadOnlyRepository;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::{Authorizer, Context, Entities, EntityUid, Request};
use log::{debug, info};
use std::sync::Arc;

pub struct CedarValidationService {
    authorizer: Authorizer,
    schema_provider: Arc<dyn SchemaProvider>,
    action_repository: Arc<ActionReadOnlyRepository>,
    resource_repository: Arc<ResourceReadOnlyRepository>,
    policy_repository: Arc<PolicyReadOnlyRepository>,
}

impl CedarValidationService {
    pub fn new(
        schema_provider: Arc<dyn SchemaProvider>,
        action_repository: Arc<ActionReadOnlyRepository>,
        resource_repository: Arc<ResourceReadOnlyRepository>,
        policy_repository: Arc<PolicyReadOnlyRepository>,
    ) -> Self {
        CedarValidationService {
            authorizer: Authorizer::new(),
            schema_provider,
            action_repository,
            resource_repository,
            policy_repository,
        }
    }
}

#[async_trait]
impl ValidationService for CedarValidationService {
    async fn validate(&self, boxer_claims: BoxerClaims, request_context: RequestContext) -> Result<(), anyhow::Error> {
        let schema = self.schema_provider.get_schema(&boxer_claims).await?;
        debug!("Cedar validation schemas: {:?}", schema);

        let action = self
            .action_repository
            .get((
                boxer_claims.validator_schema_id.clone(),
                request_context.clone().try_into()?,
            ))
            .await?;

        let resource = self
            .resource_repository
            .get((
                boxer_claims.validator_schema_id.clone(),
                request_context.clone().try_into()?,
            ))
            .await?;

        let policy_set = self
            .policy_repository
            .get(boxer_claims.validator_schema_id.clone())
            .await?;

        let actor: EntityUid = boxer_claims.principal.uid();

        let entities = Entities::empty();
        let request = Request::new(actor.clone(), action.clone(), resource.clone(), Context::empty(), None)?;
        let answer = self.authorizer.is_authorized(&request, &policy_set, &entities);

        info!(
            "validation {:?} for actor {:?} action {:?} on resource {:?}",
            answer,
            actor.to_string(),
            action.to_string(),
            resource.to_string()
        );
        match answer.decision() {
            cedar_policy::Decision::Allow => Ok(()),
            cedar_policy::Decision::Deny => anyhow::bail!("Access denied"),
        }
    }
}
