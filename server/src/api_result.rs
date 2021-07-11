use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    //    #[error("The ID token provided for identifying the user is wrong, please check the OIDC provider settings to make sure the client ID and secret are configured correctly")]
//    InvalidIdToken,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("{0}")]
    LoginError(#[from] LoginError),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Anyhow(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::LoginError(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "message": self.to_string(),
        }))
    }
}

pub type ApiResult = Result<HttpResponse, ApiError>;
