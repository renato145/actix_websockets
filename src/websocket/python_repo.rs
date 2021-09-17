use super::{
    error::WebsocketError,
    message::{ClientMessage, Connect, SubSystemPart, TaskMessage, TaskPayload, WebsocketSystems},
    subsystem::WebsocketSubSystem,
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
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PythonRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl SubSystemPart for Result<serde_json::Value, PythonRepoError> {
    fn system(&self) -> Option<WebsocketSystems> {
        Some(WebsocketSystems::PythonRepo)
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

impl WebsocketSubSystem for PythonRepoSystem {
    type Error = PythonRepoError;
    type Task = Tasks;

    fn get_address(&self, id: &Uuid) -> Option<&Recipient<ClientMessage>> {
        self.sessions.get(id)
    }

    fn system(&self) -> WebsocketSystems {
        WebsocketSystems::PythonRepo
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

/// Dispatcher for task handlers
impl Handler<TaskMessage> for PythonRepoSystem {
    type Result = ();

    #[tracing::instrument(name = "Handle task (PythonRepoSystem)", skip(self, ctx))]
    fn handle(&mut self, task: TaskMessage, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        // let task = self.task_from_message(task);
        let task_name = match serde_json::from_str::<Tasks>(&format!("{:?}", task.name))
            .context("Failed to deserialize task name.")
            .map_err(WebsocketError::MessageParseError)
        {
            Ok(task) => task,
            Err(e) => {
                if let Some(id) = task.payload.id {
                    self.send_message(id, Err(PythonRepoError::UnexpectedError(e.into())));
                }
                return;
            }
        };

        match task_name {
            Tasks::GetFiles => {
                if let Ok(task) = GetFiles::try_from(task.payload) {
                    addr.do_send(task);
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tasks {
    GetFiles,
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
        let id = payload.id.ok_or_else(|| {
            WebsocketError::MessageParseError(anyhow::anyhow!("No `id` found on payload."))
        })?;
        let path = payload
            .data
            .as_str()
            .ok_or_else(|| {
                WebsocketError::MessageParseError(anyhow::anyhow!("No `path` found on payload."))
            })?
            .into();
        Ok(Self { id, path })
    }
}

impl Handler<GetFiles> for PythonRepoSystem {
    type Result = ();

    #[tracing::instrument(name = "Handle task GetFiles", skip(self, _ctx))]
    fn handle(&mut self, message: GetFiles, _ctx: &mut Self::Context) -> Self::Result {
        if !Path::new(&message.path).exists() {
            self.send_message(message.id, Err(PythonRepoError::InvalidPath(message.path)));
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
        }
        .map_err(PythonRepoError::UnexpectedError);

        self.send_message(message.id, result);
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
