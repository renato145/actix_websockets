use super::{error::WSError, python_repo::PythonRepoMessage};
use crate::configuration::WebsocketSettings;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::{str::FromStr, time::Instant};

#[tracing::instrument(name = "Starting web socket", skip(req, stream, websocket_settings))]
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    websocket_settings: web::Data<WebsocketSettings>,
) -> Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(
        MainWebsocket::new(websocket_settings.as_ref()),
        &req,
        stream,
    );
    resp
}

struct MainWebsocket {
    hb: Instant,
    settings: WebsocketSettings,
}

impl MainWebsocket {
    fn new(settings: &WebsocketSettings) -> Self {
        Self {
            hb: Instant::now(),
            settings: settings.clone(),
        }
    }

    /// Sends ping to client every x seconds.
    /// Also checks heathbeats from client.
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(self.settings.heartbeat_interval, |act, ctx| {
            // Check client heartbeats
            if Instant::now().duration_since(act.hb) > act.settings.client_timeout {
                // heartbeat timed out
                tracing::info!("Websocket client heartbeat failed, disconnecting.");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    #[tracing::instrument(
        name = "Processing message",
        skip(self, ctx),
        fields(parsed_message=tracing::field::Empty)
    )]
    fn process_message(&self, text: &str, ctx: &mut ws::WebsocketContext<MainWebsocket>) {
        match text.parse::<WSMessage>() {
            Ok(msg) => {
                tracing::Span::current().record("parsed_message", &tracing::field::debug(&msg));
                match msg {
                    WSMessage::PythonRepo(path) => {
                        ctx.text(format!(
                            "Some result should be given here from path {:?}",
                            path
                        ));
                    }
                }
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                ctx.text(format!("{}", e));
            }
        }
    }
}

impl Actor for MainWebsocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MainWebsocket {
    #[tracing::instrument(
        name = "Handling websocket message",
        skip(self, item, ctx),
        fields(message=tracing::field::Empty)
    )]
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match item {
            Ok(msg) => msg,
            Err(_) => {
                ctx.stop();
                return;
            }
        };
        tracing::Span::current().record("message", &tracing::field::debug(&msg));

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                self.process_message(text.trim(), ctx);
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {
                tracing::info!("Invalid message");
                ctx.stop();
            }
        }
    }
}

/// Messages can be parsed from the form: "/<service>/<command>/<data>"
#[derive(Debug)]
pub enum WSMessage {
    PythonRepo(PythonRepoMessage),
}

impl FromStr for WSMessage {
    type Err = WSError;

    fn from_str(msg: &str) -> Result<Self, Self::Err> {
        let msg_parts = msg.splitn(2, '/').collect::<Vec<_>>();
        if msg_parts.len() != 2 {
            return Err(WSError::MsgParseError(anyhow::anyhow!(
                "Incomplete message: {:?}",
                msg
            )));
        }

        let msg = match msg_parts[0] {
            "python_repo" => Self::PythonRepo(msg_parts[1].parse()?),
            invalid_service => {
                return Err(WSError::MsgParseError(anyhow::anyhow!(
                    "Invalid service: {:?}",
                    invalid_service
                )))
            }
        };

        Ok(msg)
    }
}
