use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use color_eyre::Result;
use response::{HttpResult, IntoHttp};
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use message::{
    ApplicationCommandMessage, Message, RESP_TYPE_CHANNEL_MESSAGE_WITH_SOURCE, RESP_TYPE_PONG,
};

mod auth;
mod message;
mod response;
mod utils;

async fn interactions_handler(Json(message): Json<Message>) -> HttpResult {
    info!("received message from discord: {message:?}");
    match message {
        Message::Ping => Json(json!({ "type": RESP_TYPE_PONG })).into_http(),
        Message::ApplicationCommand(ApplicationCommandMessage { name, .. }) => {
            Json(json!(
                {
                    "type": RESP_TYPE_CHANNEL_MESSAGE_WITH_SOURCE, // Channel message with source
                    "data": { "content": format!("The bot has received your message: {name:?}") }
                }
            ))
            .into_http()
        }
    }
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn start_api() -> Result<()> {
    let app = Router::new()
        .route("/interactions", post(interactions_handler))
        .layer(middleware::from_fn(auth::verify_public_key_middleware))
        .route("/health", get(health_handler))
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    info!("listening on {}", bind_addr);
    let listener = TcpListener::bind(bind_addr).await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    utils::init_tracing()?;
    start_api().await?;
    Ok(())
}
