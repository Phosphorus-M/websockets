use serde::{Serialize, Deserialize};
use serde_inline_default::serde_inline_default;



#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub command: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub content: Option<String>
}

impl Message {

    pub fn has_enoughs_params(&self) -> bool{
        let Some(command) = &self.command else {
            return true;
        };
        let arguments = &self.arguments.clone().unwrap_or(Vec::new());

        match command.as_str() {
            "pokemon_info" => arguments.len() == 1,
            _ => true
        }
    }
}