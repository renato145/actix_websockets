use super::error::WebsocketError;
use actix::{Message, Recipient};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages accepted from server.
#[derive(Debug, Deserialize)]
pub struct WebsocketMessage {
    pub system: WebsocketSystems,
    pub task: TaskMessage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebsocketSystems {
    PythonRepo,
}

/// Messages that represent tasks.
#[derive(Debug, Clone, Deserialize, actix::Message)]
#[rtype(result = "()")]
pub struct TaskMessage {
    pub name: String,
    pub payload: TaskPayload,
}

#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub system: Option<WebsocketSystems>,
    pub success: bool,
    pub payload: serde_json::Value,
}

pub trait SubSystemPart {
    fn system(&self) -> Option<WebsocketSystems>;
}

pub trait ClientMessager: SubSystemPart {
    fn success(&self) -> bool;
    fn payload(self) -> serde_json::Value;
    fn to_message(self) -> ClientMessage
    where
        Self: Sized,
    {
        ClientMessage {
            system: self.system(),
            success: self.success(),
            payload: self.payload(),
        }
    }
}

impl<E> ClientMessager for Result<serde_json::Value, E>
where
    Result<serde_json::Value, E>: SubSystemPart,
    E: std::error::Error,
{
    fn success(&self) -> bool {
        self.is_ok()
    }

    fn payload(self) -> serde_json::Value {
        match self {
            Ok(value) => value,
            Err(e) => e.to_string().into(),
        }
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
