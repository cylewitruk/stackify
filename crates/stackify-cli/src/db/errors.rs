use color_eyre::eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadEnvironmentError {
    #[error("Environment '{env_name}' not found.")]
    NotFound { env_name: String },

    #[error("Parameter '{param_name}' is missing for service '{service_name}'.")]
    MissingParam {
        service_name: String,
        param_name: String,
    },

    #[error("Database error: {0}")]
    DbError(#[from] diesel::result::Error),

    #[error("Error loading environment: {0}")]
    Other(#[from] Report),
}
