use super::{
    error::WSError,
    message::{ClientMessage, Connect},
};
use crate::error_chain_fmt;
use actix::{Actor, AsyncContext, Handler, Message, Recipient};
use anyhow::Context;
use glob::glob;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum PythonRepoError {
    #[error("Invalid path: {0:?}")]
    InvalidPath(String),
}

impl std::fmt::Debug for PythonRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<PythonRepoError> for ClientMessage {
    fn from(e: PythonRepoError) -> Self {
        Self(serde_json::json!({"success": false, "payload": format!("{}", e)}))
    }
}

pub struct PythonRepoServer {
    sessions: HashMap<Uuid, Recipient<ClientMessage>>,
}

impl Default for PythonRepoServer {
    fn default() -> Self {
        Self {
            sessions: Default::default(),
        }
    }
}

impl PythonRepoServer {
    #[tracing::instrument(name = "Sending message from PythonRepoServer", skip(self))]
    pub fn send_message(&self, id: Uuid, msg: ClientMessage) {
        if let Some(addr) = self.sessions.get(&id) {
            if let Err(e) = addr.do_send(msg) {
                tracing::error!("Failed to send message from PythonRepoServer: {:?}", e);
            }
        }
    }
}

impl Actor for PythonRepoServer {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for PythonRepoServer {
    type Result = ();

    #[tracing::instrument(name = "Connecting socket to Python Repo server", skip(self, _ctx))]
    fn handle(&mut self, message: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(message.id, message.addr);
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub enum PythonRepoMessage {
    GetFiles(GetFiles),
}

impl PythonRepoMessage {
    pub fn parse(id: Uuid, message: &str) -> Result<Self, WSError> {
        let msg_parts = message.splitn(2, '/').collect::<Vec<_>>();
        match msg_parts[0] {
            "get_files" => {
                if msg_parts.len() == 2 {
                    Ok(Self::GetFiles(GetFiles {
                        id,
                        path: msg_parts[1].into(),
                    }))
                } else {
                    Err(WSError::MsgParseError("Path not given".into()))
                }
            }
            "" => Err(WSError::MsgParseError("No command given".into())),
            invalid_command => Err(WSError::MsgParseError(format!(
                "Invalid command: {:?}",
                invalid_command
            ))),
        }
    }
}

/// Dispatcher for task handlers
impl Handler<PythonRepoMessage> for PythonRepoServer {
    type Result = ();

    #[tracing::instrument(name = "Handle Python Repo message", skip(self, ctx))]
    fn handle(&mut self, message: PythonRepoMessage, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        match message {
            PythonRepoMessage::GetFiles(task) => addr.do_send(task),
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GetFiles {
    id: Uuid,
    path: String,
}

impl Handler<GetFiles> for PythonRepoServer {
    type Result = ();

    #[tracing::instrument(name = "Handle task GetFiles", skip(self, _ctx))]
    fn handle(&mut self, message: GetFiles, _ctx: &mut Self::Context) -> Self::Result {
        if !Path::new(&message.path).exists() {
            self.send_message(
                message.id,
                PythonRepoError::InvalidPath(message.path).into(),
            );
            return;
        }

        let result = match glob(&format!("{}/**/*.py", message.path))
            .context("Failed to perform glob on path.")
        {
            Ok(files) => {
                let files = files.filter_map(Result::ok).collect::<Vec<_>>();
                serde_json::to_value(files).context("Failed to convert message to JSON format.")
            }
            Err(e) => Err(e),
        };
        self.send_message(message.id, result.into());
    }
}
