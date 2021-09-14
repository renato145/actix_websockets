use super::{error::WSError, python_repo::PythonRepoMessage};
use actix::{Message, Recipient};
use uuid::Uuid;

/// Messages accepted from server.
/// Messages can be parsed from the form: "/<service>/<command>/<data>"
#[derive(Debug)]
pub enum WSMessage {
    PythonRepo(PythonRepoMessage),
}

impl WSMessage {
    pub fn parse(id: Uuid, msg: &str) -> Result<Self, WSError> {
        let msg_parts = msg.splitn(2, '/').collect::<Vec<_>>();
        if msg_parts.len() != 2 {
            return Err(WSError::MsgParseError(format!(
                "Incomplete message: {:?}",
                msg
            )));
        }

        let msg = match msg_parts[0] {
            "python_repo" => Self::PythonRepo(PythonRepoMessage::parse(id, msg_parts[1])?),
            invalid_service => {
                return Err(WSError::MsgParseError(format!(
                    "Invalid service: {:?}",
                    invalid_service
                )))
            }
        };

        Ok(msg)
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
