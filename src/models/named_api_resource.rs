use serde::Deserialize;


#[derive(Debug, Deserialize, Clone)]
pub struct NamedAPIResource {
    name: String,
    url: String,
}