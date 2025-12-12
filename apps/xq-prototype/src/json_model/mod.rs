use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonAttachment {
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonTextNote {
    pub id: String,
    pub content: String,
    #[serde(rename = "attributedTo")]
    pub author_id: String,
    #[serde(rename = "published")]
    pub created_at: String, // ISO 8601 string in JSON usually
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "type")]
    pub type_: String,
    
    // Additional fields
    pub tag: Vec<String>, // simplified tags for bench
    pub attachment: Vec<JsonAttachment>,
    #[serde(rename = "to")]
    pub to: Vec<String>, // mentions + public
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonCreateActivity {
    pub id: String,
    pub actor: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub object: JsonTextNote,
    #[serde(rename = "@context")]
    pub context: String,
}

impl Default for JsonAttachment {
    fn default() -> Self {
        Self {
            type_: "Document".to_string(),
            url: "".to_string(),
            media_type: "image/jpeg".to_string(),
        }
    }
}

impl Default for JsonTextNote {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            content: "".to_string(),
            author_id: "".to_string(),
            created_at: "".to_string(),
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            type_: "Note".to_string(),
            tag: vec![],
            attachment: vec![],
            to: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
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
