use actix::Recipient;
use uuid::Uuid;

use super::message::{ClientMessage, ClientMessager};

pub trait WebsocketSubSystem {
    type Error;

    fn get_address(&self, id: &Uuid) -> Option<&Recipient<ClientMessage>>;

    #[tracing::instrument(
		name = "Sending message from SubSystem",
		skip(self),
		fields(subsystem=tracing::field::Empty)
	)]
    fn send_message(&self, id: Uuid, msg: Result<serde_json::Value, Self::Error>)
    where
        Result<serde_json::Value, Self::Error>: ClientMessager,
        Self::Error: std::error::Error,
    {
        let type_name = std::any::type_name::<Self>();
        tracing::Span::current().record("subsystem", &tracing::field::debug(type_name));
        match self.get_address(&id) {
            Some(addr) => {
                let message = msg.to_message();
                if let Err(e) = addr.do_send(message) {
                    tracing::error!("Failed to send message from PythonRepoSystem: {:?}", e);
                }
            }
            None => {
                tracing::error!("No address found for id: {:?}", id);
            }
        }
    }
}
