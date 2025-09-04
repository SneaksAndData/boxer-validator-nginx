use crate::models::request_context::RequestContext;
use crate::services::base::schema_provider::SchemaProvider;
use crate::services::base::validation_service::ValidationService;
use crate::services::repositories::lookup_trie::backend::AssociatedRepository;
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::models::request_segment::RequestSegment;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use boxer_core::services::audit::authorization_audit_event::AuthorizationAuditEvent;
use boxer_core::services::audit::AuditService;
use boxer_core::services::observability::open_telemetry::tracing::start_trace;
use cedar_policy::{Authorizer, Context, Entities, EntityUid, PolicySet, Request};
use log::{debug, info};
use opentelemetry::context::FutureExt;
use std::sync::Arc;

pub struct CedarValidationService {
    authorizer: Authorizer,
    schema_provider: Arc<dyn SchemaProvider>,
    action_repository: Arc<AssociatedRepository<(String, Vec<RequestSegment>), EntityUid>>,
    resource_repository: Arc<AssociatedRepository<(String, Vec<PathSegment>), EntityUid>>,
    policy_repository: Arc<AssociatedRepository<String, PolicySet>>,
    audit: Arc<dyn AuditService>,
}

impl CedarValidationService {
    pub fn new(
        schema_provider: Arc<dyn SchemaProvider>,
        action_repository: Arc<AssociatedRepository<(String, Vec<RequestSegment>), EntityUid>>,
        resource_repository: Arc<AssociatedRepository<(String, Vec<PathSegment>), EntityUid>>,
        policy_repository: Arc<AssociatedRepository<String, PolicySet>>,
        audit: Arc<dyn AuditService>,
    ) -> Self {
        CedarValidationService {
            authorizer: Authorizer::new(),
            schema_provider,
            action_repository,
            resource_repository,
            policy_repository,
            audit,
        }
    }
}

#[async_trait]
impl ValidationService for CedarValidationService {
    async fn validate(&self, boxer_claims: BoxerClaims, request_context: RequestContext) -> Result<(), anyhow::Error> {
        let ctx = start_trace("request_validation");
        let schema = self
            .schema_provider
            .get_schema(&boxer_claims)
            .with_context(ctx.clone())
            .await?;
        debug!("Cedar validation schemas: {:?}", schema);

        let action = self
            .action_repository
            .get((
                boxer_claims.validator_schema_id.clone(),
                request_context.clone().try_into()?,
            ))
            .with_context(ctx.clone())
            .await?;

        let resource = self
            .resource_repository
            .get((
                boxer_claims.validator_schema_id.clone(),
                request_context.clone().try_into()?,
            ))
            .with_context(ctx.clone())
            .await?;

        let policy_set = self
            .policy_repository
            .get(boxer_claims.validator_schema_id.clone())
            .with_context(ctx.clone())
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

        self.audit
            .record_authorization(AuthorizationAuditEvent::new(&actor, &action, &resource, &answer))?;

        match answer.decision() {
            cedar_policy::Decision::Allow => Ok(()),
            cedar_policy::Decision::Deny => anyhow::bail!("Access denied"),
        }
    }
}
