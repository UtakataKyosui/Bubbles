use bytes::Bytes;
use markov::Chain;
use prost::Message;
use rand::prelude::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use xq_prototype::bubbles_xq::{Attachment, CreateActivity, TextNote};
use xq_prototype::json_model::{JsonAttachment, JsonCreateActivity, JsonTextNote};

const NUM_SERVERS: usize = 20;
const SIMULATION_DURATION_SECS: u64 = 10;
const MSGS_PER_SERVER_PER_SEC: u64 = 1000; // かなり高負荷

// 統計情報
struct Stats {
    total_messages_sent: AtomicUsize,
    total_messages_received: AtomicUsize,
    total_bytes_sent: AtomicUsize,
    total_serialize_time_ns: AtomicUsize,
    total_deserialize_time_ns: AtomicUsize,
}

impl Stats {
    fn new() -> Self {
        Self {
            total_messages_sent: AtomicUsize::new(0),
            total_messages_received: AtomicUsize::new(0),
            total_bytes_sent: AtomicUsize::new(0),
            total_serialize_time_ns: AtomicUsize::new(0),
            total_deserialize_time_ns: AtomicUsize::new(0),
        }
    }
}

// テキスト生成器
struct TextGenerator {
    chain: Chain<String>,
}

impl TextGenerator {
    fn new() -> Self {
        let mut chain = Chain::new();
        // 学習データ（適当な日本語テキスト）
        let texts = vec![
            "吾輩は猫である。名前はまだ無い。",
            "どこで生れたかとんと見当がつかぬ。",
            "何でも薄暗いじめじめした所でニャーニャー泣いていた事だけは記憶している。",
            "吾輩はここで始めて人間というものを見た。",
            "しかもあとで聞くとそれは書生という人間中で一番獰悪な種族であったそうだ。",
            "この書生というのは時々我々を捕えて煮て食うという話である。",
            "しかしその当時はそんな考もなかったから別段恐しいとも思わなかった。",
            "ただ彼の掌に載せられてスーと持ち上げられた時何だかフワフワした感じがあったばかりである。",
            "掌の上で少し落ちついて書生の顔を見たのがいわゆる人間というものの見始であろう。",
            "この時妙なものだと思った感じが今でも残っている。",
            "第一毛をもって装飾されべきはずの顔がつるつるしてまるで薬缶だ。",
            "その後猫にもだいぶ逢ったがこんな片輪には一度も出くわした事がない。",
            "のみならず顔の真中があまりに突起している。",
            "そうしてその穴の中から時々ぷうぷうと煙を吹く。",
            "どうも咽せぽくて実に弱った。",
            "これが人間の飲む煙草というものである事はようやくこの頃知った。",
            "Bubblesは分散型SNSです。",
            "ActivityPubとXQプロトコルをサポートします。",
            "高速で軽量な通信を目指しています。",
            "Protocol Buffersを使うとデータサイズが小さくなります。",
            "Rust言語で書かれています。",
            "Tauriを使ってデスクトップアプリを作っています。",
        ];
        
        // 文字単位で空白を入れて食わせることで、文字ベースのマルコフ連鎖にする
        for text in texts {
            let spaced_text: String = text.chars().map(|c| c.to_string()).collect::<Vec<String>>().join(" ");
            chain.feed_str(&spaced_text);
        }
        Self { chain }
    }

    fn generate(&self) -> String {
        let generated = self.chain.generate_str();
        // 空白を除去して戻す
        generated.replace(" ", "")
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Json,
    Protobuf,
}

#[tokio::main]
async fn main() {
    println!("Preparing benchmark...");
    let text_gen = Arc::new(TextGenerator::new());
    
    // JSON Benchmark
    println!("\n=== Starting JSON Benchmark ===");
    run_simulation(Mode::Json, text_gen.clone()).await;

    // Protobuf Benchmark
    println!("\n=== Starting Protobuf Benchmark ===");
    run_simulation(Mode::Protobuf, text_gen.clone()).await;
}

async fn run_simulation(mode: Mode, text_gen: Arc<TextGenerator>) {
    let stats = Arc::new(Stats::new());
    let mut senders = Vec::new();
    let mut receivers = Vec::new();

    for _ in 0..NUM_SERVERS {
        let (tx, rx) = mpsc::channel::<Bytes>(10000); // バッファ多めに
        senders.push(tx);
        receivers.push(rx);
    }

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for i in 0..NUM_SERVERS {
        let my_rx = receivers.pop().unwrap();
        let all_senders = senders.clone();
        let stats_clone = stats.clone();
        let text_gen_clone = text_gen.clone();
        let server_id = format!("server-{}", i);

        handles.push(tokio::spawn(async move {
            run_server(server_id, my_rx, all_senders, stats_clone, mode, text_gen_clone).await;
        }));
    }

    // Run for duration
    tokio::time::sleep(Duration::from_secs(SIMULATION_DURATION_SECS)).await;

    // Report
    let total_sent = stats.total_messages_sent.load(Ordering::Relaxed);
    let total_received = stats.total_messages_received.load(Ordering::Relaxed);
    let total_bytes = stats.total_bytes_sent.load(Ordering::Relaxed);
    let total_ser_time = stats.total_serialize_time_ns.load(Ordering::Relaxed);
    let total_deser_time = stats.total_deserialize_time_ns.load(Ordering::Relaxed);

    println!("Duration: {}s", SIMULATION_DURATION_SECS);
    println!("Total Messages Sent: {}", total_sent);
    println!("Message Rate: {:.2} msg/sec", total_sent as f64 / SIMULATION_DURATION_SECS as f64);
    println!("Total Data Transferred: {:.2} MB", total_bytes as f64 / 1024.0 / 1024.0);
    println!("Data Rate: {:.2} MB/sec", (total_bytes as f64 / 1024.0 / 1024.0) / SIMULATION_DURATION_SECS as f64);
    
    if total_sent > 0 {
        println!("Avg Serialize Time: {:.2} ns", total_ser_time as f64 / total_sent as f64);
    }
    if total_received > 0 {
        println!("Avg Deserialize Time: {:.2} ns", total_deser_time as f64 / total_received as f64);
    }
}

async fn run_server(
    id: String,
    mut rx: mpsc::Receiver<Bytes>,
    peer_txs: Vec<mpsc::Sender<Bytes>>,
    stats: Arc<Stats>,
    mode: Mode,
    text_gen: Arc<TextGenerator>,
) {
    let interval_micros = 1_000_000 / MSGS_PER_SERVER_PER_SEC;
    let mut timer = tokio::time::interval(Duration::from_micros(interval_micros));
    // let mut rng = rand::rng(); // Removed from here

    loop {
        tokio::select! {
            _ = timer.tick() => {
                // Generate and Send
                let content = text_gen.generate();
                let now_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let msg_id = format!("{}-{}", id, now_ts);
                
                // 対象サーバーをランダムに選択
                let mut rng = rand::rng(); // Scope locally to ensure Send
                let target_indices: Vec<usize> = (0..peer_txs.len())
                    .filter(|&_idx| peer_txs.len() > 1) 
                    .choose_multiple(&mut rng, 3);

                let bytes = if mode == Mode::Protobuf {
                    let note = TextNote {
                        id: format!("note-{}", msg_id),
                        content: content.clone(),
                        author_id: format!("user-{}", id),
                        created_at: now_ts as i64,
                        tags: vec!["#bubbles".to_string(), "#xq".to_string()],
                        attachments: vec![Attachment { 
                            r#type: "Image".to_string(), 
                            url: "https://example.com/img.jpg".to_string(), 
                            media_type: "image/jpeg".to_string() 
                        }],
                        mentions: vec![],
                    };
                    let activity = CreateActivity {
                        id: format!("activity-{}", msg_id),
                        actor_id: format!("user-{}", id),
                        note: Some(note),
                    };
                    
                    let start = Instant::now();
                    let mut buf = Vec::with_capacity(512);
                    activity.encode(&mut buf).unwrap();
                    stats.total_serialize_time_ns.fetch_add(start.elapsed().as_nanos() as usize, Ordering::Relaxed);
                    Bytes::from(buf)
                } else {
                    let note = JsonTextNote {
                        id: format!("note-{}", msg_id),
                        content: content.clone(),
                        author_id: format!("user-{}", id),
                        created_at: now_ts.to_string(),
                        tag: vec!["#bubbles".to_string(), "#xq".to_string()],
                        attachment: vec![JsonAttachment { 
                            type_: "Image".to_string(), 
                            url: "https://example.com/img.jpg".to_string(), 
                            media_type: "image/jpeg".to_string() 
                        }],
                        ..Default::default()
                    };
                    let activity = JsonCreateActivity {
                        id: format!("activity-{}", msg_id),
                        actor: format!("user-{}", id),
                        object: note,
                        ..Default::default()
                    };
                    
                    let start = Instant::now();
                    let json = serde_json::to_string(&activity).unwrap();
                    stats.total_serialize_time_ns.fetch_add(start.elapsed().as_nanos() as usize, Ordering::Relaxed);
                    Bytes::from(json)
                };

                let len = bytes.len();
                for &idx in &target_indices {
                    if let Some(tx) = peer_txs.get(idx) {
                        let _ = tx.try_send(bytes.clone()); // 詰まっていたらドロップ（現実に近い）
                        stats.total_messages_sent.fetch_add(1, Ordering::Relaxed);
                        stats.total_bytes_sent.fetch_add(len, Ordering::Relaxed);
                    }
                }
            }
            Some(bytes) = rx.recv() => {
                // Receive and Deserialize
                let start = Instant::now();
                if mode == Mode::Protobuf {
                   let _ = CreateActivity::decode(bytes);
                } else {
                   let _ : Result<JsonCreateActivity, _> = serde_json::from_slice(&bytes);
                }
                stats.total_deserialize_time_ns.fetch_add(start.elapsed().as_nanos() as usize, Ordering::Relaxed);
                stats.total_messages_received.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
