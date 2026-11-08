pub mod handlers {
    use axum::{Router, routing::get, Json, extract::State};
    use crate::{AppState, auth::AuthUser, common::response::ApiResponse};
    pub fn router() -> Router<AppState> { Router::new().route("/", get(list)) }
    async fn list(State(_s): State<AppState>, _user: AuthUser) -> Json<ApiResponse<Vec<serde_json::Value>>> { Json(ApiResponse::new(vec![])) }
}
