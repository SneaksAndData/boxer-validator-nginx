/// Boxer's validation configuration
pub struct ValidationSettings {
   pub secret: &'static str,
   pub valid_issuers: &'static Vec<&'static str>, 
   pub valid_audiences: &'static Vec<&'static str>,
}
