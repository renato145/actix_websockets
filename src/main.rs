use actix::{Actor, StreamHandler};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_websockets::telemetry::{get_subscriber, init_subscriber};
use tracing_actix_web::TracingLogger;

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
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    tracing::info!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("actix_websockets".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/ws", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
