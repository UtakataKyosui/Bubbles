use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MODEL_NAME: &str = "meta-llama/llama-3-8b-instruct";

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

// Internal structure for the SLM's JSON output
#[derive(Debug, Deserialize, Clone)]
pub struct AnalysisResult {
    pub is_misinformation: bool,
    pub confidence_score: f32,
    pub label_type: String, // "MISINFORMATION", "DISINFORMATION", "SATIRE", "NONE"
    pub reasoning_summary: String,
    pub suggested_evidence_keywords: Vec<String>,
}

pub struct AutoLabeler {
    api_key: String,
    client: reqwest::Client,
}

impl AutoLabeler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| "OPENROUTER_API_KEY environment variable not set")?;
            
        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
        })
    }

    pub async fn analyze_text(&self, text: &str) -> Result<AnalysisResult, Box<dyn Error>> {
        let system_prompt = r#"You are an automated fact-checking assistant for a social network.
Analyze the following user post for potential misinformation, disinformation, or harmful content.
Focus on objective verifiability, usage of strong emotional language, and lack of sources.

Return your analysis in the following JSON format ONLY:
{
  "is_misinformation": boolean,
  "confidence_score": float (0.0-1.0),
  "label_type": "MISINFORMATION" | "DISINFORMATION" | "SATIRE" | "NONE",
  "reasoning_summary": "Short explanation (max 100 chars)",
  "suggested_evidence_keywords": ["keyword1", "keyword2"]
}"#;

        let request_body = ChatCompletionRequest {
            model: MODEL_NAME.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            // Note: Not all OpenRouter models support response_format: {"type": "json_object"} yet, 
            // but many newer ones do. If not supported, we rely on the prompt instructions.
            response_format: Some(ResponseFormat { type_: "json_object".to_string() }),
        };

        let res = self.client.post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://bubbles-network.app") // OpenRouter requirement
            .header("X-Title", "Bubbles Fact Checker")
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
             return Err(format!("API request failed: {}", res.status()).into());
        }

        let response_data: ChatCompletionResponse = res.json().await?;
        
        if let Some(choice) = response_data.choices.first() {
            let content = &choice.message.content;
            // Attempt to parse the JSON content from the LLM
            let analysis: AnalysisResult = serde_json::from_str(content)?;
            Ok(analysis)
        } else {
            Err("No choices returned from API".into())
        }
    }
}
