use crate::helpers::spawn_app;

#[actix_rt::test]
async fn receive_python_files_on_valid_path() {
    // Arrange
    let app = spawn_app().await;
    let message = serde_json::json!({
        "system": "python_repo",
        "task": {
            "name": "get_files",
            "payload": {"data": "tests/examples"}
        }
    })
    .to_string();

    // Act
    let result = app.get_first_result(&message).await;

    // Assert
    let success = result.get("success").unwrap().as_bool().unwrap();
    assert!(success, "Call was not successful.");
    let payload = result.get("payload").unwrap().to_string();
    assert!(
        payload.contains("a.py"),
        "Expected file (a.py) not found in payload."
    );
}

#[actix_rt::test]
async fn receive_error_on_invalid_path() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let msg = app
        .get_first_result("python_repo/get_files/tests/some_incorrect_path")
        .await;

    // Assert
    let success = msg.get("success").unwrap().as_bool().unwrap();
    assert!(!success, "Call should not success.");
}
