use crate::helpers::spawn_app;
use awc::Client;

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;

    // Act
    tracing::info!("==> Trying {}/ws/", app.address);
    let (response, framed) = Client::new()
        .ws(format!("{}/ws/", app.address))
        .connect()
        .await
        .expect("Failed to connect to websocket.");

    // let a = framed.split();

    // connection.
    //     .send(awc::ws::Message::Text("Echo".to_string()))
    //     .await.unwrap();

    tracing::info!("==> {:?}", response);
    // Assert
}
