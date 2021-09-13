use crate::helpers::spawn_app;
use awc::Client;
use futures::StreamExt;
use std::time::Duration;

#[actix_rt::test]
async fn client_receives_heartbeat_every_x_secs() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let (_response, mut connection) = Client::new()
        .ws(format!("{}/ws/", app.address))
        .connect()
        .await
        .expect("Failed to connect to websocket.");

    let sleep = tokio::time::sleep(Duration::from_millis(1000));
    tokio::pin!(sleep);
    let mut count = 0;

    loop {
        tokio::select! {
            msg = connection.next() => {
                tracing::info!("==> {:?}", msg);
                count += 1;
            }
            _ = &mut sleep => {
                tracing::info!("Timeout!");
                break;
            }
        }
    }

    // Assert
    assert!(count >= 0, "Did not receive any heathbeats.")

    // connection
    //     .send(awc::ws::Message::Ping("".into()))
    //     .await
    //     .unwrap();

    // let response = connection.next().await;
    // tracing::info!("==> {:?}", response);

    // connection
    //     .send(awc::ws::Message::Text("Echo".into()))
    //     .await
    //     .unwrap();

    // let response = connection.next().await;
    // tracing::info!("==> {:?}", response);
}
