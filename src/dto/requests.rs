use axum::{response::IntoResponse, Json};

pub async fn request() -> impl IntoResponse {
    let message = "hello word";

    let json_response = serde_json::json!({
        "status":"success",
        "message": message
    });

    Json(json_response)
}
