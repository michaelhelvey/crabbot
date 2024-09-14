//! Module for parsing discord messages and forming responses
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

// Interaction types on requests:
pub const INT_TYPE_PING: u8 = 1;
pub const INT_TYPE_APP_CMD: u8 = 2;

// Interaction types on responses:
pub const RESP_TYPE_PONG: u8 = 1;
pub const RESP_TYPE_CHANNEL_MESSAGE_WITH_SOURCE: u8 = 4;

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ApplicationCommandType {
    ChatInput = 1,
    User,
    Message,
    PrimaryEntryPoint,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ApplicationCommandOptionType {
    SubCommand = 1,
    SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
    Attachment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionDataOption {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: ApplicationCommandOptionType,
    pub value: Option<Value>,
    pub options: Option<Vec<ApplicationCommandInteractionDataOption>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationCommandMessage {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: ApplicationCommandType,
    pub options: Option<Vec<ApplicationCommandInteractionDataOption>>,
}

#[derive(Debug)]
pub enum Message {
    Ping,
    // See: https://discord.com/developers/docs/interactions/receiving-and-responding
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
        let helper = MessageHelper::deserialize(deserializer)?;
        match helper.r#type {
            INT_TYPE_PING => Ok(Message::Ping),
            INT_TYPE_APP_CMD => {
                let data: ApplicationCommandMessage =
                    serde_json::from_value(helper.data).map_err(serde::de::Error::custom)?;
                Ok(Message::ApplicationCommand(data))
            }
            _ => Err(serde::de::Error::custom("unknown message type")),
        }
    }
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

        let Message::ApplicationCommand(ApplicationCommandMessage { name, .. }) = message else {
            panic!("was not ApplicationCommand");
        };

        assert_eq!(name, "test");
    }
}
