use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    jwt::jwt_auth::JWTAuthMiddleware,
    model::todo_model::{CreateTodoSchema, SelectTodo, Todo, UpdateTodoSchema},
    // model::user_model::{LoginUserSchema, RegisterUserSchema, User},
    AppState,
};

pub async fn create_todo(
    State(data): State<Arc<AppState>>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    Json(body): Json<CreateTodoSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if body.name.is_empty() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Todo name is required"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    if body.description.is_empty() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Todo description is required"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // The `body.name` and `body.description` fields are guaranteed to be non-empty
    // since they are of type `String`.

    let todo = sqlx::query_as!(
        Todo,
        "INSERT INTO todos (user_id, name, description) VALUES ($1, $2, $3) RETURNING id, user_id, name, description, created_at, updated_at",
        jwtauth.user.id,
        body.name,
        body.description
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "todo": todo
        })
    });

    Ok(Json(json_response))
}

pub async fn get_todo(
    State(data): State<Arc<AppState>>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let todo = sqlx::query_as!(
        Todo,
        "SELECT *
        FROM todos
        WHERE user_id = $1",
        jwtauth.user.id,
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "todo": todo
        })
    });

    Ok(Json(json_response))
}

pub async fn updated_todo(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    Json(body): Json<UpdateTodoSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("Handler started"); // Add this for debugging

    // Fetch the existing TODO item from the database
    let existing_todo = sqlx::query_as!(
        SelectTodo,
        "SELECT id, name, description FROM todos WHERE user_id = $1 AND id = $2",
        jwtauth.user.id,
        id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Check if the TODO item exists
    let mut existing_todo = match existing_todo {
        Some(todo) => todo,
        None => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "TODO item not found",
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    };

    // Apply partial updates from the request body
    if let Some(name) = body.name {
        existing_todo.name = name;
    }

    if let Some(description) = body.description {
        existing_todo.description = description;
    }

    // Update the TODO item in the database with the applied updates
    sqlx::query!(
        "UPDATE todos SET name = $1, description = $2 WHERE user_id = $3 AND id = $4",
        existing_todo.name,
        existing_todo.description,
        jwtauth.user.id,
        id
    )
    .execute(&data.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "todo": existing_todo
        })
    });

    Ok(Json(json_response))
}

pub async fn delete_todo(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("Handler started"); // Add this for debugging

    // Check if the TODO item exists and belongs to the user.
    let todo_exists = sqlx::query_as!(
        Todo,
        "SELECT * FROM todos WHERE user_id = $1 AND id = $2",
        &jwtauth.user.id,
        &id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error checking TODO item: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Failed to check TODO item"
            })),
        )
    })?;

    if todo_exists.is_none() {
        // The TODO item doesn't exist or doesn't belong to the user.
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": "TODO item not found"
            })),
        ));
    }

    // Delete the TODO item from the database.
    if let Err(e) = sqlx::query("DELETE FROM todos WHERE user_id = $1 AND id = $2")
        .bind(&jwtauth.user.id)
        .bind(&id)
        .execute(&data.db)
        .await
    {
        eprintln!("Error deleting TODO item: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Failed to delete TODO item"
            })),
        ));
    }

    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "message": "TODO item deleted successfully".to_string(),
        })
    });

    // Return a success response.
    Ok(Json(json_response))
}
