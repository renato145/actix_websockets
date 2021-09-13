use super::error::WSError;
use std::str::FromStr;

#[derive(Debug)]
pub enum PythonRepoMessage {
    GetFiles(String),
}

impl FromStr for PythonRepoMessage {
    type Err = WSError;

    fn from_str(msg: &str) -> Result<Self, Self::Err> {
        let msg_parts = msg.splitn(2, '/').collect::<Vec<_>>();
        match msg_parts[0] {
            "get_files" => {
                if msg_parts.len() == 2 {
                    Ok(Self::GetFiles(msg_parts[1].into()))
                } else {
                    Err(WSError::MsgParseError(anyhow::anyhow!("Path not given")))
                }
            }
            "" => Err(WSError::MsgParseError(anyhow::anyhow!("No command given"))),
            invalid_command => Err(WSError::MsgParseError(anyhow::anyhow!(
                "Invalid command: {:?}",
                invalid_command
            ))),
        }
    }
}

pub struct PythonRepoServer;
