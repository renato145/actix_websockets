use super::{
    error::WebsocketError,
    message::{ClientMessage, Connect, TaskMessage, TaskPayload},
};
use crate::error_chain_fmt;
use actix::{Actor, AsyncContext, Handler, Message, Recipient};
use anyhow::Context;
use glob::glob;
use serde::Deserialize;
use std::{collections::HashMap, convert::TryFrom, path::Path};
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

pub struct PythonRepoSystem {
    sessions: HashMap<Uuid, Recipient<ClientMessage>>,
}

impl Default for PythonRepoSystem {
    fn default() -> Self {
        Self {
            sessions: Default::default(),
        }
    }
}

impl PythonRepoSystem {
    #[tracing::instrument(name = "Sending message from PythonRepoSystem", skip(self))]
    pub fn send_message(&self, id: Uuid, msg: ClientMessage) {
        if let Some(addr) = self.sessions.get(&id) {
            if let Err(e) = addr.do_send(msg) {
                tracing::error!("Failed to send message from PythonRepoSystem: {:?}", e);
            }
        }
    }
}

impl Actor for PythonRepoSystem {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for PythonRepoSystem {
    type Result = ();

    #[tracing::instrument(name = "Connecting socket to PythonRepoSystem", skip(self, _ctx))]
    fn handle(&mut self, message: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(message.id, message.addr);
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tasks {
    GetFiles,
}

/// Dispatcher for task handlers
impl Handler<TaskMessage> for PythonRepoSystem {
    type Result = Result<(), WebsocketError>;

    #[tracing::instrument(name = "Handle task (PythonRepoSystem)", skip(self, ctx))]
    fn handle(&mut self, task: TaskMessage, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        match serde_json::from_str::<Tasks>(&format!("{:?}", task.name))
            .context("Failed to deserialize task name.")
            .map_err(WebsocketError::MessageParseError)?
        {
            Tasks::GetFiles => {
                addr.do_send(GetFiles::try_from(task.payload)?);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GetFiles {
    id: Uuid,
    path: String,
}

impl TryFrom<TaskPayload> for GetFiles {
    type Error = WebsocketError;

    fn try_from(payload: TaskPayload) -> Result<Self, Self::Error> {
        let id = payload
            .id
            .ok_or(WebsocketError::MessageParseError(anyhow::anyhow!(
                "No `id` found on payload."
            )))?;
        let path = payload
            .data
            .as_str()
            .ok_or(WebsocketError::MessageParseError(anyhow::anyhow!(
                "No `path` found on payload."
            )))?
            .into();
        Ok(Self { id, path })
    }
}

impl Handler<GetFiles> for PythonRepoSystem {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket::message::WebsocketMessage;

    #[test]
    fn correctly_deserialize_task_name() {
        let message = serde_json::json!({
            "system": "python_repo",
            "task": {
                "name": "get_files",
                "payload": {"data": "tests/examples"}
            }
        });
        let message = serde_json::from_value::<WebsocketMessage>(message).unwrap();
        let task = serde_json::from_str::<Tasks>(&format!("{:?}", message.task.name)).unwrap();
        assert_eq!(Tasks::GetFiles, task);
    }
}
