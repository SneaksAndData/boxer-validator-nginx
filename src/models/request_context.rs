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
}
