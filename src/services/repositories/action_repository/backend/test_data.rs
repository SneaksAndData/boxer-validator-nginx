use serde_json::json;
pub fn test_routes() -> serde_json::Value {
    json!({
        "hostname": "api.example.com",
        "routes": [
            {
                "method": "Get",
                "route_template": "/resources/{resourceId}/",
                "action_uid": "TestApp::\"ReadResource\""
            },
            {
                "method": "Post",
                "route_template": "/resources/{resourceId}/",
                "action_uid": "TestApp::\"CreateResource\""
            },
            {
                "method": "Post",
                "route_template": "/resources/{resourceId}/watchers",
                "action_uid": "TestApp::\"CreateWatcher\""
            },
            {
                "method": "Post",
                "route_template": "/resources/{resourceId}/watchers/{watcherId}/watch",
                "action_uid": "TestApp::\"WatchResource\""
            }
        ]
    })
}

pub fn test_updated_routes() -> serde_json::Value {
    json!({
        "hostname": "api.v2.example.com",
        "routes": [
            {
                "method": "Get",
                "route_template": "/api/v1/items/{resourceId}/",
                "action_uid": "TestApp::\"ReadResource\""
            },
            {
                "method": "Post",
                "route_template": "/api/v1/items/{resourceId}/",
                "action_uid": "TestApp::\"CreateResource\""
            },
            {
                "method": "Post",
                "route_template": "/api/v1/items/{resourceId}/watchers",
                "action_uid": "TestApp::\"CreateWatcher\""
            },
            {
                "method": "Post",
                "route_template": "/api/v1/items/{resourceId}/watchers/{watcherId}/watch",
                "action_uid": "TestApp::\"WatchResource\""
            }
        ]
    })
}
