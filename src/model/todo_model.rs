use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Todo {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SelectTodo {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoSchema {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoSchema {
    pub name: Option<String>,
    pub description: Option<String>,
}
