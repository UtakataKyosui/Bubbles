use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct FactChecker {
    trusted_bots: Vec<PublicKey>,
}

impl FactChecker {
    pub fn new(bots: Vec<PublicKey>) -> Self {
        Self { trusted_bots: bots }
    }

    pub fn is_trusted_bot(&self, pubkey: &PublicKey) -> bool {
        self.trusted_bots.contains(pubkey)
    }

    // Check if an event has a label from a trusted bot
    pub fn verify_label(&self, label_event: &Event) -> Option<String> {
        if !self.is_trusted_bot(&label_event.pubkey) {
            return None;
        }
        
        if label_event.kind != Kind::Label { // Kind 1985
            return None;
        }

        // Extract label content
        // Looking for ["l", "value", ...]
        for tag in label_event.tags.iter() {
            let t = tag.as_slice();
            if t.len() >= 2 && t[0] == "l" {
                 return Some(t[1].clone());
            }
        }
        
        None
    }
}
