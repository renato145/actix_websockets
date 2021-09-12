use crate::helpers::spawn_app;
use awc::Client;
use futures::{SinkExt, StreamExt};

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;

    // Act
    // let (response, framed) = Client::new()
    let (response, mut connection) = Client::new()
        .ws(format!("{}/ws/", app.address))
        .connect()
        .await
        .expect("Failed to connect to websocket.");
    tracing::info!("==> {:?}", response);

    connection
        .send(awc::ws::Message::Ping("".into()))
        .await
        .unwrap();

    let response = connection.next().await;
    tracing::info!("==> {:?}", response);

    connection
        .send(awc::ws::Message::Text("Echo".into()))
        .await
        .unwrap();

    let response = connection.next().await;
    tracing::info!("==> {:?}", response);
    // Assert
}
