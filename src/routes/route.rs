use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    dto::requests::request,
    handler::{
        handler::{
            get_me_handler, health_checker_handler, login_user_handler, logout_handler,
            refresh_access_token_handler, register_user_handler,
        },
        todohandler::{create_todo, delete_todo, get_todo, updated_todo},
    },
    jwt::jwt_auth::auth,
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .route("/", get(request))
        .route("/healthchecker", get(health_checker_handler))
        .route("/auth/register", post(register_user_handler))
        .route("/auth/login", post(login_user_handler))
        .route("/auth/refresh", get(refresh_access_token_handler))
        .route(
            "/auth/logout",
            get(logout_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/users/me",
            get(get_me_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/todo",
            post(create_todo).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/todo",
            get(get_todo).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/todo/:id",
            patch(updated_todo)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/todo/:id",
            delete(delete_todo)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    Router::new().nest("/api", api_route)
}
