use std::str::FromStr;
use anyhow::Error;
use cedar_policy::{Authorizer, Context, Entities, EntityId, EntityTypeName, EntityUid, PolicySet, Request};
use futures_util::future::ok;
use log::info;

pub trait ValidationService {
    fn validate(&self, token: &str) -> Result<(), anyhow::Error>;
}

pub struct CedarValidationService {
    authorizer: Authorizer
}

impl CedarValidationService {
    pub fn new() -> Self {
        CedarValidationService {
            authorizer: Authorizer::new(),
        }
    }
}

impl ValidationService for CedarValidationService{
    fn validate_token(&self, policy: &str) -> Result<(), Error> {
        const POLICY_SRC: &str = r#"
            permit(
                principal == User::AzureAD::"visa@ecco.com",
                action == Action::"GET",
                resource
            );
"#;
        let policy: PolicySet = POLICY_SRC.parse().unwrap();

        let action = r#"Action::"view""#.parse().unwrap();
        let alice = r#"User::"alice""#.parse().unwrap();
        let file = r#"File::"93""#.parse().unwrap();
        let request = Request::new(Some(alice), Some(action), Some(file), Context::empty(), None).unwrap();

        let entities = Entities::empty();
        let authorizer = Authorizer::new();
        let answer = authorizer.is_authorized(&request, &policy, &entities);

        // Should output `Allow`
        info!("validation {:?}", answer.decision());

        let action = r#"Action::"GET""#.parse().unwrap();
        let bob = r#"User::AzureAD::"visa@ecco.com""#.parse().unwrap();
        let tp = EntityTypeName::from_str("URI").unwrap();
        let id = EntityId::from_str("https://crystal.sneaksanddata.com/run/omni-channel-solver").unwrap();
        let file = EntityUid::from_type_name_and_id(tp, id);
        // file.att
        info!("file {:?}", file);
        // file["id"] = "https://crystal.sneaksanddata.com/run/omni-channel-solver";
        let request = Request::new(Some(bob), Some(action), Some(file), Context::empty(), None).unwrap();

        let answer = authorizer.is_authorized(&request, &policy, &entities);

        // Should output `Deny`
        info!("validation {:?}", answer.decision());
        
        let action = r#"Action::"GET""#.parse().unwrap();
        let bob = r#"User::GoogleAD::"visa@ecco.com""#.parse().unwrap();
        let file = r#"URI::"https://crystal.sneaksanddata.com/run/omni-channel-solver""#.parse().unwrap();
        let request = Request::new(Some(bob), Some(action), Some(file), Context::empty(), None).unwrap();

        let answer = authorizer.is_authorized(&request, &policy, &entities);

        // Should output `Deny`
        info!("validation {:?}", answer.decision());
        
        Ok(())
    }
}
