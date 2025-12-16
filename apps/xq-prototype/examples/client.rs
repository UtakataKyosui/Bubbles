use bytes::Bytes;
use prost::Message;
use reqwest::header;
use std::time::{SystemTime, UNIX_EPOCH};
use xq_prototype::bubbles_xq::{CreateActivity, PostActivityRequest, PostActivityResponse, TextNote};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // データの作成
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let note = TextNote {
        id: "note-123".to_string(),
        content: "Hello, XQ World! This is a test message via Protobuf.".to_string(),
        author_id: "user-456".to_string(),
        created_at: now,
    };

    let activity = CreateActivity {
        id: "activity-789".to_string(),
        actor_id: "user-456".to_string(),
        note: Some(note),
    };

    let request = PostActivityRequest {
        activity: Some(activity),
    };

    // Protobufエンコード
    let mut buf = Vec::new();
    request.encode(&mut buf)?;

    println!("Sending request to http://localhost:3000/api/activity...");
    
    // HTTPクライアント作成して送信
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:3000/api/activity")
        .header(header::CONTENT_TYPE, "application/x-protobuf")
        .body(buf)
        .send()?;

    println!("Response Status: {}", res.status());

    // レスポンスの処理
    if res.status().is_success() {
        let body_bytes = res.bytes()?;
        let response = PostActivityResponse::decode(body_bytes)?;
        
        println!("Response decoded successfully:");
        println!("  Success: {}", response.success);
        println!("  Message: {}", response.message);
        println!("  Activity ID: {}", response.activity_id);
    } else {
        println!("Request failed: {:?}", res.text()?);
    }

    Ok(())
}
