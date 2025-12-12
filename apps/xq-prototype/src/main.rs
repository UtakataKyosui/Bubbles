use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use bytes::Bytes;
use prost::Message;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use xq_prototype::bubbles_xq::{PostActivityRequest, PostActivityResponse};

#[tokio::main]
async fn main() {
    // ログ設定
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // ルーター設定
    let app = Router::new()
        .route("/api/activity", post(handle_activity));

    // サーバー起動
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_activity(req: Request<Body>) -> impl IntoResponse {
    // Content-Typeのチェック
    let content_type = req.headers().get(header::CONTENT_TYPE);
    
    // BodyをBytesとして取得
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to read body: {}", e),
            ).into_response();
        }
    };

    // Protobufデコード
    let request: PostActivityRequest = match PostActivityRequest::decode(body_bytes) {
        Ok(req) => req,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to decode protobuf: {}", e),
            ).into_response();
        }
    };

    // リクエスト内容の処理
    if let Some(activity) = &request.activity {
        info!("Received Activity: ID={}", activity.id);
        info!("Actor: {}", activity.actor_id);
        if let Some(note) = &activity.note {
            info!("Note Content: {}", note.content);
        }
    } else {
        info!("Received empty activity");
    }

    // レスポンス作成
    let response = PostActivityResponse {
        success: true,
        message: "Activity received successfully".to_string(),
        activity_id: request.activity.map(|a| a.id).unwrap_or_default(),
    };

    // Protobufエンコード
    let mut buf = Vec::new();
    if let Err(e) = response.encode(&mut buf) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode response: {}", e),
        ).into_response();
    }

    // レスポンスとして返す
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-protobuf")],
        Bytes::from(buf),
    ).into_response()
}
