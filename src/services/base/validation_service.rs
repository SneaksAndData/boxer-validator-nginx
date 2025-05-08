use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::models::request_context::RequestContext;

pub trait ValidationService {
    fn validate(
        &self,
        boxer_claims: BoxerClaims,
        request_context: RequestContext,
    ) -> Result<(), anyhow::Error>;
}
