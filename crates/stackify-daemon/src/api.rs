use rocket::{get, routes, serde::json::Json};
use serde_json::json;

use crate::models::GetStatusResponse;

pub type Result<T> = color_eyre::Result<Json<T>, crate::errors::ApiError>;

pub fn routes() -> Vec<rocket::Route> {
    routes![get_status]
}

#[get("/status")]
pub async fn get_status() -> Json<GetStatusResponse> {
    let response = GetStatusResponse {
        status: "ok".to_string(),
    };

    Json(response)
}