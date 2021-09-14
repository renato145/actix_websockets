use crate::error_chain_fmt;

use super::message::ClientMessage;

#[derive(thiserror::Error)]
pub enum WebsocketError {
    #[error("Failed to parse websocket message.")]
    MessageParseError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for WebsocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<WebsocketError> for ClientMessage {
    fn from(e: WebsocketError) -> Self {
        Self(serde_json::json!({"success": false, "payload": format!("{}", e)}))
    }
}
