use crate::models::request_context::RequestContext;
use crate::services::prefix_tree::bucket::hash_bucket::HashTrieBucket;
use crate::services::prefix_tree::bucket::request_segment_bucket::RequestBucket;
use crate::services::prefix_tree::hash_tree::HashTrie;
use crate::services::prefix_tree::MutableTrie;
use crate::services::repositories::models::http_method::HTTPMethod;
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::models::request_segment::RequestSegment;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test_case("api/v1/resources/resource1" => matches Some("value1"); "simple insert")]
#[test_case("" => matches None; "insert empty key")]
#[tokio::test]
async fn test_insert(key: &str) -> Option<&'static str> {
    let mut trie = HashTrie::<HashTrieBucket<u8, &str>>::new();
    HashTrie::insert(&mut trie, key, "value1").await;
    trie.get(key).await
}

#[test_case(("api/v1/resources/resource1", "api/v1/resources/resource1") => matches Some("value1"); "exact match")]
#[test_case(("api/v1/resources/resource1", "api/v1/resources/resource") => matches None; "partial match")]
#[test_case(("", "api/v1/resources/") => matches None; "empty key with query")]
#[test_case(("", "") => matches None; "empty key with empty query")]
#[tokio::test]
async fn test_partial_query(keys: (&str, &str)) -> Option<&'static str> {
    let (key, query) = keys;
    let mut trie = HashTrie::<HashTrieBucket<u8, &str>>::new();
    trie.insert(key, "value1").await;
    trie.get(query).await
}

#[tokio::test]
async fn test_overwrite_existing_value() {
    let key = "api/v1/resources/resource1";
    let mut trie = HashTrie::<HashTrieBucket<u8, &str>>::new();
    trie.insert(key, "value1").await;
    trie.insert(key, "value2").await;
    trie.insert(key, "value3").await;
    let value = trie.get(key).await.expect("Expected to find the key in the trie");
    assert_eq!(value, "value3");
}

fn wrapped_pretty_assert(expected: String) -> impl Fn(String) {
    move |actual: String| assert_eq!(actual, expected)
}

#[test_case("www.example.com/api/v1/resources/resource/resource1" => using wrapped_pretty_assert("value0".to_string()); "with parameter match")]
#[test_case("www.example.com/api/v1/resources/my-resource" => using wrapped_pretty_assert("value1".to_string()); "with exact match")]
//  TODO: Not implemented since sample with my-id below overwrites the sample with parameter
// #[test_case("www.example.com/api/v1/resources/my-resource/ids/an-id" => using wrapped_pretty_assert("value2".to_string()); "with deeper parameter in path")]
#[test_case("www.example.com/i-do-not/know/what/i-am/doing" => using wrapped_pretty_assert("value3".to_string()); "with full parameter path")]
#[test_case("www.example.com/api/v1/resources/my-resource/ids/my-id" => using wrapped_pretty_assert("value4".to_string()); "with full exact path in the end")]
#[tokio::test]
async fn test_path_segment_matchers(key: &str) -> String {
    let segments = vec![
        // www.example.com/api/v1/resources/resource/{id}
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Static("resource".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        // www.example.com/api/v1/resources/{resource_id}
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        // Note: this value is not actually correct, but it still must be handled by the trie
        // www.example.com/api/v1/resources/{resource_id}/ids/{id}
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Static("ids".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        // Note: this value is not actually correct, but it still must be handled by the trie
        // www.example.com/{parameter}/{parameter}/{parameter}/{resource_id}/{id}
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        // Note: this value is not actually correct, but it still must be handled by the trie
        // www.example.com/api/v1/resources/my-resource/ids/my-id
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Static("my-resource".to_string())),
            RequestSegment::Path(PathSegment::Static("ids".to_string())),
            RequestSegment::Path(PathSegment::Static("my-id".to_string())),
        ],
    ];
    let mut trie = HashTrie::<RequestBucket<String>>::new();

    for i in 0..segments.len() {
        trie.insert(segments[i].clone(), format!("value{}", i)).await;
    }

    let rc: Vec<RequestSegment> = RequestContext::new(format!("http://{key}"), "GET".to_string())
        .try_into()
        .unwrap();

    println!("Request Context Segments: {:?}", rc);
    trie.get(&rc).await.unwrap_or("".to_string())
}
