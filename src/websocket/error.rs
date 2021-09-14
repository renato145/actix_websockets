use crate::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum WebsocketError {
    #[error("Failed to parse websocket message: {0}")]
    MsgParseError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for WebsocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
