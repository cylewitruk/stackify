use rocket::{get, post, routes, serde::json::Json};
use stackify_common::api::models::{GetStatusResponse, Status, UpdateConfigRequest};

use crate::errors::ApiError;

pub fn routes() -> Vec<rocket::Route> {
    routes![get_status]
}

#[get("/status")]
pub async fn get_status() -> std::result::Result<Json<GetStatusResponse>, ApiError> {
    let response = GetStatusResponse {
        status: Status::Ok,
        services: Default::default(),
    };

    Ok(Json(response))
}

#[get("/start-service")]
pub async fn start_service() -> std::result::Result<Json<()>, ApiError> {
    Ok(Json(()))
}

#[post("/stop-service")]
pub async fn stop_service() -> std::result::Result<Json<()>, ApiError> {
    Ok(Json(()))
}

#[post("/restart-service")]
pub async fn restart_service() -> std::result::Result<Json<()>, ApiError> {
    Ok(Json(()))
}

#[post("/update-config", data = "<_req>")]
pub async fn update_config(
    _req: Json<UpdateConfigRequest>,
) -> std::result::Result<Json<()>, ApiError> {
    Ok(Json(()))
}
