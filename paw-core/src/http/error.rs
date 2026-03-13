use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

pub type ApiResult<T> = Result<T, ApiError>;
pub type HttpClientError = ApiError;
pub type HttpErrorKind = ApiErrorKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    Server,
    Network,
    Timeout,
    Client,
    InvalidResponse,
    Unknown,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{message}")]
pub struct ApiError {
    status_code: Option<u16>,
    code: Option<String>,
    message: String,
    kind: ApiErrorKind,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ErrorPayload {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl ApiError {
    fn new(
        status_code: Option<u16>,
        code: Option<String>,
        message: impl Into<String>,
        kind: ApiErrorKind,
    ) -> Self {
        Self {
            status_code,
            code,
            message: message.into(),
            kind,
        }
    }

    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::new(None, None, message, ApiErrorKind::InvalidResponse)
    }

    pub fn unauthorized(message: impl Into<String>, code: Option<String>) -> Self {
        Self::new(
            Some(StatusCode::UNAUTHORIZED.as_u16()),
            code,
            message,
            ApiErrorKind::Unauthorized,
        )
    }

    pub fn forbidden(message: impl Into<String>, code: Option<String>) -> Self {
        Self::new(
            Some(StatusCode::FORBIDDEN.as_u16()),
            code,
            message,
            ApiErrorKind::Forbidden,
        )
    }

    pub fn not_found(message: impl Into<String>, code: Option<String>) -> Self {
        Self::new(
            Some(StatusCode::NOT_FOUND.as_u16()),
            code,
            message,
            ApiErrorKind::NotFound,
        )
    }

    pub fn server(status: StatusCode, message: impl Into<String>, code: Option<String>) -> Self {
        Self::new(Some(status.as_u16()), code, message, ApiErrorKind::Server)
    }

    pub fn client(status: StatusCode, message: impl Into<String>, code: Option<String>) -> Self {
        Self::new(Some(status.as_u16()), code, message, ApiErrorKind::Client)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(None, None, message, ApiErrorKind::Network)
    }

    pub fn timeout() -> Self {
        Self::new(None, None, "Request timed out", ApiErrorKind::Timeout)
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(None, None, message, ApiErrorKind::Unknown)
    }

    pub fn status_code(&self) -> Option<u16> {
        self.status_code
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn kind(&self) -> ApiErrorKind {
        self.kind
    }

    pub fn is_unauthorized(&self) -> bool {
        self.kind == ApiErrorKind::Unauthorized
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::timeout();
        }
        if error.is_decode() {
            return Self::invalid_response(error.to_string());
        }
        Self::network(error.to_string())
    }
}
