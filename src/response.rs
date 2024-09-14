use axum::{
    response::{IntoResponse, Response},
    Json,
};
use color_eyre::Report;
use http::{header, StatusCode};
use serde_json::json;

pub type HttpResult = Result<Response, HttpError>;

pub trait IntoHttp {
    fn into_http(self) -> HttpResult;
}

impl<T: IntoResponse> IntoHttp for T {
    fn into_http(self) -> HttpResult {
        Ok(self.into_response())
    }
}

#[derive(Debug)]
pub enum HttpError {
    Internal { err: String },
}

impl HttpError {
    fn from_report(err: Report) -> Self {
        HttpError::Internal {
            err: err.to_string(),
        }
    }
}

macro_rules! impl_from {
    ($from:ty) => {
        impl From<$from> for HttpError {
            fn from(err: $from) -> Self {
                Self::from_report(err.into())
            }
        }
    };
}

impl_from!(std::io::Error);
impl_from!(color_eyre::Report);
impl_from!(serde_json::Error);
impl_from!(axum::Error);

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::Internal { err } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())],
                Json(json!({ "error": err.to_string() })),
            )
                .into_response(),
        }
    }
}
