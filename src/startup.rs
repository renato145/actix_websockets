use crate::{
    configuration::{Settings, WebsocketSettings},
    websocket::{python_repo::PythonRepoServer, route::ws_index},
};
use actix::Actor;
use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let address = format!("{}:{}", configuration.host, configuration.port);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, configuration.websocket)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    websocket_settings: WebsocketSettings,
) -> Result<Server, std::io::Error> {
    tracing::info!("{:?}", websocket_settings);
    let websocket_settings = Data::new(websocket_settings);
    let python_repo_server = Data::new(PythonRepoServer::default().start());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/ws/", web::get().to(ws_index))
            .app_data(websocket_settings.clone())
            .app_data(python_repo_server.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
