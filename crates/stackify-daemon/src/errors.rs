use std::io::Cursor;

use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[error("{}", self)]
pub enum ApiError {
    General(String),
}

impl<'o: 'r, 'r> Responder<'r, 'o> for ApiError {
    fn respond_to(
        self, 
        _: &'r rocket::Request<'_>
    ) -> rocket::response::Result<'o> {
        let serialized = serde_json::to_string(&self).unwrap();

        let response = rocket::response::Response::build()
            .status(rocket::http::Status::InternalServerError)
            .header(rocket::http::ContentType::JSON)
            .sized_body(serialized.len(), Cursor::new(serialized))
            .finalize();

        Ok(response)
    }
}