use std::str::FromStr;
use boxer_core::services::base::upsert_repository::UpsertRepository;
use cedar_policy::{EntityId, EntityTypeName, EntityUid};
use crate::models::request_context::RequestContext;
use crate::services::action_repository::{ActionMapping, ActionRepository};

#[tokio::test]
async fn test_insert_and_get_action_mapping() {
    let r = RequestContext::new("https://example.com/api/resource".to_string(), "GET".to_string());
    let action_repo = ActionRepository::new();
    let action_mapping: ActionMapping = r.try_into().unwrap();


    let entity_type = EntityTypeName::from_str("TestApp").unwrap();
    let entity_id = EntityId::from_str("RunAction").unwrap();
    let entity_uid = EntityUid::from_type_name_and_id(entity_type, entity_id);

    action_repo.upsert(action_mapping.clone(), entity_uid).await.unwrap();
    let action = action_repo.get(action_mapping.clone()).await.unwrap();

    assert_eq!("TestApp::\"RunAction\"", action.to_string());
}
