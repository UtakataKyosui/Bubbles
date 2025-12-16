use prost::Message;
use std::time::{SystemTime, UNIX_EPOCH};
use xq_prototype::bubbles_xq::{CreateActivity, TextNote};
use xq_prototype::json_model::{JsonCreateActivity, JsonTextNote};

fn main() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // Setup Protobuf Data
    let proto_note = TextNote {
        id: "note-1234567890".to_string(),
        content: "Hello, world! This is a benchmark for XQ (Protobuf) vs ActivityPub (JSON). We are testing serialization speed and payload size.".to_string(),
        author_id: "user-1234567890".to_string(),
        created_at: now,
    };
    let proto_activity = CreateActivity {
        id: "activity-1234567890".to_string(),
        actor_id: "user-1234567890".to_string(),
        note: Some(proto_note),
    };

    // Setup JSON Data
    let json_note = JsonTextNote {
        id: "note-1234567890".to_string(),
        content: "Hello, world! This is a benchmark for XQ (Protobuf) vs ActivityPub (JSON). We are testing serialization speed and payload size.".to_string(),
        author_id: "user-1234567890".to_string(),
        created_at: now,
        ..Default::default()
    };
    let json_activity = JsonCreateActivity {
        id: "activity-1234567890".to_string(),
        actor: "user-1234567890".to_string(),
        object: json_note,
        ..Default::default()
    };

    // Print Sizes
    let mut proto_buf = Vec::new();
    proto_activity.encode(&mut proto_buf).unwrap();
    let json_string = serde_json::to_string(&json_activity).unwrap();
    
    println!("\n--- Payload Size Comparison ---");
    println!("Content: 'Hello, world! This is a benchmark for XQ (Protobuf) vs ActivityPub (JSON). We are testing serialization speed and payload size.'");
    println!("Protobuf Size: {} bytes", proto_buf.len());
    println!("JSON Size:     {} bytes", json_string.len());
    let reduction = (1.0 - (proto_buf.len() as f64 / json_string.len() as f64)) * 100.0;
    println!("Reduction:     {:.2}%", reduction);
    println!("-------------------------------\n");
}
