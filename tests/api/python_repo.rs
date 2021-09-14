use crate::helpers::spawn_app;
use actix_web_actors::ws;
use awc::Client;
use futures::{SinkExt, StreamExt};

#[actix_rt::test]
async fn receive_python_files_on_valid_path() {
    // Arrange
    let app = spawn_app().await;

    let (_response, mut connection) = Client::new()
        .ws(format!("{}/ws/", app.address))
        .connect()
        .await
        .expect("Failed to connect to websocket.");

    // Act
    connection
        .send(awc::ws::Message::Text(
            "python_repo/get_files/tests/examples".into(),
        ))
        .await
        .expect("Failed to send message.");

    // Assert
    if let Some(Ok(ws::Frame::Text(msg))) = connection.next().await {
        let msg = serde_json::from_slice::<serde_json::Value>(&msg).expect("Failed to parse JSON.");
        tracing::info!("{}", msg);
        let success = msg.get("success").unwrap().as_bool().unwrap();
        assert!(success, "Call was not successful.");
        let payload = msg.get("payload").unwrap().to_string();
        assert!(
            payload.contains("a.py"),
            "Expected file (a.py) not found in payload."
        );
    } else {
        panic!("Failed to receive message.");
    }
}

#[actix_rt::test]
async fn receive_error_on_invalid_path() {
    // Arrange
    let app = spawn_app().await;

    let (_response, mut connection) = Client::new()
        .ws(format!("{}/ws/", app.address))
        .connect()
        .await
        .expect("Failed to connect to websocket.");

    // Act
    connection
        .send(awc::ws::Message::Text(
            "python_repo/get_files/tests/some_incorrect_path".into(),
        ))
        .await
        .expect("Failed to send message.");

    // Assert
    if let Some(Ok(ws::Frame::Text(msg))) = connection.next().await {
        let msg = serde_json::from_slice::<serde_json::Value>(&msg).expect("Failed to parse JSON.");
        tracing::info!("{}", msg);
        let success = msg.get("success").unwrap().as_bool().unwrap();
        assert!(!success, "Call should not success.");
    } else {
        panic!("Failed to receive message.");
    }
}
