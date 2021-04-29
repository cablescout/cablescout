use actix_web::body::Body;
use actix_web::dev::BaseHttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{BaseHttpResponse, HttpResponse, ResponseError};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("The ID token provided for identifying the user is wrong, please check the OIDC provider settings to make sure the client ID and secret are configured correctly")]
    InvalidIdToken,
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

    fn error_response(&self) -> BaseHttpResponse<Body> {
        let body = match serde_json::to_string(&json!({
            "message": self.to_string(),
        })) {
            Ok(body) => body,
            Err(err) => {
                log::error!("Failed formatting error to json ({}): {}", err, self);
                "\"An error has occurred but could not be formatted to JSON\"".to_owned()
            }
        };
        BaseHttpResponseBuilder::new(self.status_code())
            .content_type(mime::APPLICATION_JSON)
            .body(body)
    }
}

pub type ApiResult = Result<HttpResponse, ApiError>;
