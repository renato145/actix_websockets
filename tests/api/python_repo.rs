use crate::helpers::spawn_app;


#[actix_rt::test]
async fn retrieve_python_files_on_valid_path() {
	// Arrange
	let app = spawn_app().await;

	// Act
    // let (_response, mut connection) = Client::new()
    //     .ws(format!("{}/ws/", app.address))
    //     .connect()
    //     .await
    //     .expect("Failed to connect to websocket.");

    // let sleep = tokio::time::sleep(Duration::from_millis(250));
    // tokio::pin!(sleep);
    // let mut count = 0;


	// Assert
	todo!()
}

#[actix_rt::test]
async fn send_error_on_invalid_path() {
	todo!()
}
