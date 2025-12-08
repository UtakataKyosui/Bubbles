use nostr_sdk::prelude::*;
use std::time::Duration;
use thiserror::Error;

use crate::wot::WotGraph;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Nostr SDK error: {0}")]
    Nostr(#[from] nostr_sdk::client::Error),
    #[error("Keys error: {0}")]
    Keys(#[from] nostr_sdk::key::Error),
    #[error("Tag parse error: {0}")]
    TagParse(String),
}

#[derive(Clone)]
pub struct BubbleClient {
    client: Client,
}

impl BubbleClient {
    pub async fn new(keys: Option<Keys>) -> Result<Self, ClientError> {
        let keys = if let Some(k) = keys {
            k
        } else {
            // Try to load from bubble_keys.json
            if let Ok(file) = std::fs::File::open("bubble_keys.json") {
                let reader = std::io::BufReader::new(file);
                if let Ok(secret_key) = serde_json::from_reader::<_, String>(reader) {
                     Keys::parse(&secret_key)?
                } else {
                    let k = Keys::generate();
                    let file = std::fs::File::create("bubble_keys.json").unwrap(); // TODO: handle error
                    serde_json::to_writer(file, &k.secret_key().to_secret_hex()).unwrap();
                    k
                }
            } else {
                let k = Keys::generate();
                let file = std::fs::File::create("bubble_keys.json").unwrap(); // TODO: handle error
                serde_json::to_writer(file, &k.secret_key().to_secret_hex()).unwrap();
                k
            }
        };
        
        let client = Client::new(keys);
        
        // Add default relays
        client.add_relay("wss://relay.damus.io").await?;
        client.add_relay("wss://nos.lol").await?;
        
        client.connect().await;
        
        Ok(Self { client })
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }

    pub async fn get_own_pubkey(&self) -> PublicKey {
        self.client.signer().await.unwrap().get_public_key().await.unwrap()
    }

    pub async fn publish_text_note(&self, content: &str) -> Result<EventId, ClientError> {
        let builder = EventBuilder::text_note(content);
        let output = self.client.send_event_builder(builder).await?;
        Ok(*output.id())
    }

    pub async fn get_timeline(&self, limit: usize) -> Result<Vec<Event>, ClientError> {
        let filter = Filter::new()
            .kind(Kind::TextNote)
            .limit(limit);
        
        // Use fetch_events instead of get_events_of
        let events = self.client.fetch_events(vec![filter], Some(Duration::from_secs(10))).await?;
        Ok(events.to_vec())
    }

    pub async fn fetch_contacts(&self, pubkey: PublicKey) -> Result<Vec<PublicKey>, ClientError> {
        let filter = Filter::new()
            .kind(Kind::ContactList)
            .author(pubkey)
            .limit(1);
            
        let events = self.client.fetch_events(vec![filter], Some(Duration::from_secs(10))).await?;
        
        if let Some(event) = events.first() {
             let mut contacts = Vec::new();
             for tag in event.tags.iter() {
                 let t = tag.as_slice();
                 if t.len() >= 2 && t[0] == "p" {
                     if let Ok(pk) = PublicKey::parse(&t[1]) {
                         contacts.push(pk);
                     }
                 }
             }
             Ok(contacts)
        } else {
            Ok(vec![])
        }
    }

    pub async fn publish_label(&self, target_id: EventId, label: String) -> Result<EventId, ClientError> {
        let e_tag = Tag::parse(["e", &target_id.to_string()]).map_err(|_| ClientError::TagParse("e tag".to_string()))?;
        let l_tag = Tag::parse(["l", &label, "bubble"]).map_err(|_| ClientError::TagParse("l tag".to_string()))?;
        
        let builder = EventBuilder::new(Kind::Label, "").tags(vec![e_tag, l_tag]);
        let output = self.client.send_event_builder(builder).await?;
        Ok(*output.id())
    }

    pub async fn fetch_labels(&self, target_id: EventId) -> Result<Vec<String>, ClientError> {
        let filter = Filter::new()
            .kind(Kind::Label)
            .event(target_id);
            
        let events = self.client.fetch_events(vec![filter], Some(Duration::from_secs(10))).await?;
        
        let mut labels = Vec::new();
        for event in events {
            for tag in event.tags.iter() {
                 let t = tag.as_slice();
                 if t.len() >= 2 && t[0] == "l" {
                     labels.push(t[1].clone());
                 }
            }
        }
        Ok(labels)
    }

    pub async fn build_wot(&self, root: PublicKey, depth: u8) -> Result<WotGraph, ClientError> {
        let mut graph = WotGraph::new();
        
        let mut current_layer = vec![root];
        let mut visited = std::collections::HashSet::new();
        visited.insert(root);

        for _d in 0..depth {
            if current_layer.is_empty() {
                break;
            }
            
            let mut next_layer_all: Vec<PublicKey> = Vec::new();
            
            // Limit to 200 authors per batch to be safe
            for chunk in current_layer.chunks(200) {
                 let filter = Filter::new()
                    .kind(Kind::ContactList)
                    .authors(chunk.to_vec())
                    .limit(chunk.len()); 
                 
                 let events = self.client.fetch_events(vec![filter], Some(Duration::from_secs(10))).await?;
                 
                 for event in events {
                    let author = event.pubkey;
                    for tag in event.tags.iter() {
                         let t = tag.as_slice();
                         if t.len() >= 2 && t[0] == "p" {
                             if let Ok(target) = PublicKey::parse(&t[1]) {
                                 graph.add_link(author, target);
                                 if visited.insert(target) {
                                     next_layer_all.push(target);
                                 }
                             }
                         }
                    }
                 }
            }
            current_layer = next_layer_all;
        }
        
        Ok(graph)
    }
}
