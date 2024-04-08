use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadEnvironmentError {
    #[error("Environment not found.")]
    NotFound,

    #[error("Parameter '{param_name}' is missing for service '{service_name}'.")]
    MissingParam {
        service_name: String,
        param_name: String,
    },

    #[error("Database error: {0}")]
    DbError(#[from] diesel::result::Error),
}
