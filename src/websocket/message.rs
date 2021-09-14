use super::error::WebsocketError;
use actix::{Message, Recipient};
use anyhow::Context;
use serde::Deserialize;
use uuid::Uuid;

/// Messages accepted from server.
#[derive(Debug, Deserialize)]
pub struct WebsocketMessage {
    pub system: WebsocketSystems,
    pub task: TaskMessage,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebsocketSystems {
    PythonRepo,
}

/// Messages that represent tasks.
#[derive(Debug, Deserialize, actix::Message)]
#[rtype(result = "Result<(), WebsocketError>")]
pub struct TaskMessage {
    pub name: String,
    pub payload: TaskPayload,
}

#[derive(Debug, Deserialize)]
pub struct TaskPayload {
    #[serde(skip)]
    pub id: Option<Uuid>,
    pub data: serde_json::Value,
}

impl WebsocketMessage {
    pub fn parse(id: Uuid, message: &str) -> Result<Self, WebsocketError> {
        let mut message = serde_json::from_str::<WebsocketMessage>(message)
            .context("Failed to deserialize message.")
            .map_err(WebsocketError::MessageParseError)?;
        message.task.payload.id = Some(id);
        Ok(message)
    }
}

/// Messages to send to client.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ClientMessage(pub serde_json::Value);

impl From<Result<serde_json::Value, anyhow::Error>> for ClientMessage {
    fn from(res: Result<serde_json::Value, anyhow::Error>) -> Self {
        match res {
            Ok(value) => Self(serde_json::json!({"success": true, "payload": value})),
            Err(e) => e.into(),
        }
    }
}

impl From<anyhow::Error> for ClientMessage {
    fn from(e: anyhow::Error) -> Self {
        Self(serde_json::json!({"success": false, "payload": format!("{}", e)}))
    }
}

/// Start connection with a server.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<ClientMessage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_deserialize_websocket_message() {
        let message = serde_json::json!({
            "system": "python_repo",
            "task": {
                "name": "get_files",
                "payload": {"data": "tests/examples"}
            }
        });
        let message = serde_json::from_value::<WebsocketMessage>(message).unwrap();
        assert_eq!(WebsocketSystems::PythonRepo, message.system);
        assert_eq!(None, message.task.payload.id);
    }
}
