use crate::configuration::WebsocketSettings;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::Instant;

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
        // ctx.run_interval(dur, f)
    }
}

impl Actor for MainWebsocket {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MainWebsocket {
    #[tracing::instrument(name = "Handling web socket", skip(self, ctx))]
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

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
    tracing::info!("{:?}", resp);
    resp
}
