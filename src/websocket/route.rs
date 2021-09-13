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
}

impl Actor for MainWebsocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MainWebsocket {
    #[tracing::instrument(name = "Handling websocket message", skip(self, ctx))]
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match item {
            Ok(msg) => msg,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(reason) => {
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
    resp
}
