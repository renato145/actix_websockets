use crate::helpers::spawn_app;
use actix_web_actors::ws;
use awc::Client;
use futures::{SinkExt, StreamExt};

#[actix_rt::test]
async fn retrieve_python_files_on_valid_path() {
    // Arrange
    let app = spawn_app().await;

    let (_response, mut connection) = Client::new()
        .ws(format!("{}/ws/", app.address))
        .connect()
        .await
        .expect("Failed to connect to websocket.");

    // Act
    connection
        .send(awc::ws::Message::Text("/python_repo/".into()))
        .await
        .expect("Failed to send message.");

    if let Some(Ok(ws::Frame::Text(msg))) = connection.next().await {
        tracing::info!("==> {:?}", msg);
    }

    // let sleep = tokio::time::sleep(Duration::from_millis(250));
    // tokio::pin!(sleep);
    // let mut count = 0;

    // Assert
}

#[actix_rt::test]
async fn send_error_on_invalid_path() {
    todo!()
}
