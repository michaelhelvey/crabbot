use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use color_eyre::Result;
use response::{HttpResult, IntoHttp};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use serde_repr::Serialize_repr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

mod auth;
mod response;
mod utils;

#[derive(Debug, Serialize, Deserialize)]
struct ApplicationCommandMessage {
    name: String,
}

#[derive(Debug)]
enum Message {
    // Ping = 1
    Ping,
    // ApplicationCommand = 2
    ApplicationCommand(ApplicationCommandMessage),
}

#[derive(Debug, Deserialize)]
struct MessageHelper {
    r#type: u8,
    #[serde(default)]
    data: Value,
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        info!("deserializing");
        let helper = MessageHelper::deserialize(deserializer)?;

        info!("deserializing helper {helper:?}");
        match helper.r#type {
            1 => Ok(Message::Ping),
            2 => {
                let data: ApplicationCommandMessage =
                    serde_json::from_value(helper.data).map_err(serde::de::Error::custom)?;
                Ok(Message::ApplicationCommand(data))
            }
            _ => Err(serde::de::Error::custom("unknown message type")),
        }
    }
}

async fn interactions_handler(Json(message): Json<Message>) -> HttpResult {
    info!("received message from discord: {message:?}");
    match message {
        Message::Ping => {
            return Json(json!({ "type": 1 })).into_http();
        }
        Message::ApplicationCommand(ApplicationCommandMessage { name }) => {
            info!("received application command with name {name:?}");
            return Json(json!(
                {
                    "type": 4, // Channel message with source
                    "data": { "content": "Hi there from the bot" }
                }
            ))
            .into_http();
        }
        _ => unreachable!(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discord_message() {
        let content = r#"{
            "type": 2,
            "data": {
                "name": "test"
            }
        }"#;
        let message: Message = serde_json::from_str(content).unwrap();

        let Message::ApplicationCommand(ApplicationCommandMessage { name }) = message else {
            panic!("was not ApplicationCommand");
        };

        assert_eq!(name, "test");
    }
}
