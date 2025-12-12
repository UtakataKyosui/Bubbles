use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonTextNote {
    pub id: String,
    pub content: String,
    pub author_id: String,
    pub created_at: i64,
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonCreateActivity {
    pub id: String,
    pub actor: String, // actor_id
    #[serde(rename = "type")]
    pub type_: String,
    pub object: JsonTextNote,
    #[serde(rename = "@context")]
    pub context: String,
}

impl Default for JsonTextNote {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            content: "".to_string(),
            author_id: "".to_string(),
            created_at: 0,
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            type_: "Note".to_string(),
        }
    }
}

impl Default for JsonCreateActivity {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            actor: "".to_string(),
            type_: "Create".to_string(),
            object: JsonTextNote::default(),
            context: "https://www.w3.org/ns/activitystreams".to_string(),
        }
    }
}
