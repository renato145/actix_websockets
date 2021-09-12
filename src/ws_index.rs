use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;

// Define HTTP actor
struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
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

#[tracing::instrument(name = "Starting web socket", skip(req, stream))]
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    tracing::info!("{:?}", resp);
    resp
}
