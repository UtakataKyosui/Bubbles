use std::collections::{HashMap, HashSet};
use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct WotGraph {
    // pubkey -> set of followed pubkeys
    graph: HashMap<PublicKey, HashSet<PublicKey>>,
}

impl WotGraph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    pub fn add_link(&mut self, from: PublicKey, to: PublicKey) {
        self.graph.entry(from).or_default().insert(to);
    }
    
    pub fn compute_trust(&self, source: &PublicKey, target: &PublicKey) -> f64 {
        if source == target {
            return 1.0;
        }

        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        
        queue.push_back((source, 0));
        visited.insert(source);
        
        let max_depth = 3; // Check up to 3 hops
        
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            if let Some(follows) = self.graph.get(current) {
                if follows.contains(target) {
                    // Distance is depth + 1. 
                    // Direct follow (depth 0) -> dist 1 -> score 1.0
                    // FoF (depth 1) -> dist 2 -> score 0.5
                    return 1.0 / (depth as f64 + 1.0);
                }
                
                for next in follows {
                    if visited.insert(next) {
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
        
        0.0
    }
}
