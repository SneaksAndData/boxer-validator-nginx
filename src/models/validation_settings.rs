/// Boxer's validation configuration
#[derive(Debug, Clone)]
pub struct ValidationSettings {
   pub secret: &'static str,
   pub valid_issuers: Vec<String>, 
   pub valid_audiences: Vec<String>,
}

impl ValidationSettings {
    pub fn new() -> Self {
       ValidationSettings {
          secret: "dummy-secret",
          valid_issuers: vec!["boxer.sneaksanddata.com".to_string()],
          valid_audiences: vec!["boxer.sneaksanddata.com".to_string()]
       }
    }
}
