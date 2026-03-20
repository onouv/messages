use serde::{Serialize, Deserialize};
use derive_getters::Getters;

#[derive(Serialize, Deserialize, Getters, Clone, Debug)]
pub struct ComponentDTO {
    id: String,
    name: String,
    description: Option<String>,
}

impl ComponentDTO {
    pub fn new(id: &str, name: &str, description: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
        }
    }
}