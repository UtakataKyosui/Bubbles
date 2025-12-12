use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prost::Message;
use std::time::{SystemTime, UNIX_EPOCH};
use xq_prototype::bubbles_xq::{CreateActivity, TextNote};
use xq_prototype::json_model::{JsonCreateActivity, JsonTextNote};

fn benchmark_serialization(c: &mut Criterion) {
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
    println!("Protobuf Size: {} bytes", proto_buf.len());
    println!("JSON Size:     {} bytes", json_string.len());
    println!("Reduction:     {:.2}%", (1.0 - (proto_buf.len() as f64 / json_string.len() as f64)) * 100.0);
    println!("-------------------------------\n");

    let mut group = c.benchmark_group("serialization");
    
    group.bench_function("protobuf_serialize", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(proto_buf.len());
            black_box(&proto_activity).encode(&mut buf).unwrap();
            buf
        })
    });

    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&json_activity)).unwrap()
        })
    });
    
    group.finish();
}

fn benchmark_deserialization(c: &mut Criterion) {
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
    let mut proto_buf = Vec::new();
    proto_activity.encode(&mut proto_buf).unwrap();
    let proto_bytes = bytes::Bytes::from(proto_buf);

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
    let json_string = serde_json::to_string(&json_activity).unwrap();
    let json_bytes = json_string.as_bytes();

    let mut group = c.benchmark_group("deserialization");
    
    group.bench_function("protobuf_deserialize", |b| {
        b.iter(|| {
            black_box(CreateActivity::decode(proto_bytes.clone())).unwrap()
        })
    });

    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let res: JsonCreateActivity = black_box(serde_json::from_slice(json_bytes)).unwrap();
            res
        })
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_serialization, benchmark_deserialization);
criterion_main!(benches);
