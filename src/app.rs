use bubble_core::BubbleClient;
use bubble_core::nostr_sdk::prelude::*;
use anyhow::Result;

use bubble_core::wot::WotGraph;
use std::collections::HashMap;
use bubble_core::fact_check::FactChecker;

pub struct App {
    pub client: BubbleClient,
    pub timeline: Vec<Event>,
    pub should_quit: bool,
    pub status: String,
    pub wot: Option<WotGraph>,
    pub trust_scores: HashMap<PublicKey, f64>,
    pub fact_checker: FactChecker,
    pub verifications: HashMap<EventId, String>,
    pub input: String,
    pub input_mode: bool,
    pub scroll_state: ratatui::widgets::ListState,
}

impl App {
    pub async fn new() -> Result<Self> {
        // In a real app we would load keys from disk
        let client = BubbleClient::new(None).await?;
        
        // Setup Fact Checker with some trusted bot keys
        // For demo purposes, we trust ourself + some hardcoded bot
        let my_key = client.get_own_pubkey().await;
        // Example Hex Key (just a placeholder or derived)
        let checker = FactChecker::new(vec![my_key]);
        
        Ok(Self {
            client,
            timeline: vec![],
            should_quit: false,
            status: "Ready".to_string(),
            wot: None,
            trust_scores: HashMap::new(),
            fact_checker: checker,
            verifications: HashMap::new(),
            input: String::new(),
            input_mode: false,
            scroll_state: ratatui::widgets::ListState::default(),
        })
    }

    pub async fn publish_input(&mut self) {
        if self.input.is_empty() {
             return;
        }
        
        match self.client.publish_text_note(&self.input).await {
            Ok(_) => {
                self.status = "Published!".to_string();
                self.input.clear();
                self.input_mode = false;
                self.refresh_timeline().await;
            },
            Err(e) => {
                self.status = format!("Publish Error: {}", e);
            }
        }
    }

    pub async fn refresh_timeline(&mut self) {
        self.status = "Refreshing...".to_string();
        
        // 1. Fetch Timeline
        match self.client.get_timeline(20).await {
            Ok(events) => {
                self.timeline = events;
                
                // 2. Build WoT (Lazy load or refresh)
                // For MVP: Rebuild every time (inefficient but safe). 
                // Using depth 1.
                let my_pubkey = self.client.get_own_pubkey().await;
                match self.client.build_wot(my_pubkey, 1).await {
                    Ok(graph) => {
                        self.wot = Some(graph.clone());
                        
                        // 3. Compute Trust Scores for visible events
                        self.trust_scores.clear();
                        for event in &self.timeline {
                            let score = graph.compute_trust(&my_pubkey, &event.pubkey);
                            self.trust_scores.insert(event.pubkey, score);
                        }
                    },
                    Err(e) => {
                        self.status = format!("WoT Error: {}", e);
                    }
                }

                // 4. Fetch Verifications (Labels)
                let event_ids: Vec<EventId> = self.timeline.iter().map(|e| e.id).collect();
                if !event_ids.is_empty() {
                     let filter = Filter::new()
                        .kind(Kind::Label)
                        .events(event_ids);
                    if let Ok(label_events) = self.client.inner().fetch_events(vec![filter], Some(std::time::Duration::from_secs(5))).await {
                        self.verifications.clear();
                        for label_event in label_events {
                            // Extract target event id 'e' tag
                            let mut target_id = None;
                            for tag in label_event.tags.iter() {
                                let t = tag.as_slice();
                                if t.len() >= 2 && t[0] == "e" {
                                    if let Ok(tid) = EventId::parse(&t[1]) {
                                        target_id = Some(tid);
                                    }
                                }
                            }
                            
                            if let Some(tid) = target_id {
                                if let Some(verdict) = self.fact_checker.verify_label(&label_event) {
                                    self.verifications.insert(tid, verdict);
                                }
                            }
                        }
                    }
                }
                
                self.status = format!("Updated: {} events, WoT built, Labels checked", self.timeline.len());
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    pub fn scroll_down(&mut self) {
        if self.timeline.is_empty() { return; }
        
        let i = match self.scroll_state.selected() {
            Some(i) => {
                if i >= self.timeline.len() - 1 {
                    self.timeline.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.scroll_state.select(Some(i));
    }

    pub fn scroll_up(&mut self) {
        if self.timeline.is_empty() { return; }

        let i = match self.scroll_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.scroll_state.select(Some(i));
    }
}
