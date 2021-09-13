use crate::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum WSError {
    #[error("Failed to parse websocket message.")]
    MsgParseError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for WSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}