use serde::de::DeserializeOwned;
use std::fmt;

/// Alby error response from 400 and 500 status codes.
#[derive(Debug, serde::Deserialize)]
pub struct ErrorResponse {
    /// Alby error code.
    pub code: u32,
    /// Indicates if it is an error.
    pub error: bool,
    /// Alby error message.
    pub message: String,
}

/// Alby API request error.
#[derive(Debug)]
pub enum RequestError {
    /// Unexpected error.
    Unexpected(String),
    /// Failed to create auth header.
    AuthHeaderCreation(reqwest::header::InvalidHeaderValue),
    /// Failed to create reqwest client.
    ClientCreation(reqwest::Error),
    /// Failed to send request.
    RequestSend(reqwest::Error),
    /// Failed to read response body.
    ResponseBodyRead(reqwest::Error),
    /// Failed to parse response body.
    ResponseParse(serde_json::Error, String),
    /// Bad request (400).
    BadRequest(ErrorResponse),
    /// Internal server error (500).
    InternalServerError(ErrorResponse),
    /// Unexpected status code.
    UnexpectedStatus {
        /// Status code.
        status: reqwest::StatusCode,
        /// Response body.
        body: String,
    },
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Unexpected(e) => write!(f, "Unexpected error: {}", e),
            RequestError::AuthHeaderCreation(e) => write!(f, "Failed to create auth header: {}", e),
            RequestError::ClientCreation(e) => write!(f, "Failed to create reqwest client: {}", e),
            RequestError::RequestSend(e) => write!(f, "Failed to send request: {}", e),
            RequestError::ResponseBodyRead(e) => write!(f, "Failed to read response body: {}", e),
            RequestError::ResponseParse(e, body) => {
                write!(f, "Failed to parse response body ({}): {}", body, e)
            }
            RequestError::BadRequest(e) => {
                write!(f, "Bad request (400): {} (code: {})", e.message, e.code)
            }
            RequestError::InternalServerError(e) => write!(
                f,
                "Internal server error (500): {} (code: {})",
                e.message, e.code
            ),
            RequestError::UnexpectedStatus { status, body } => {
                write!(f, "Unexpected status code: {}. Body: {}", status, body)
            }
        }
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RequestError::Unexpected(_) => None,
            RequestError::AuthHeaderCreation(e) => Some(e),
            RequestError::ClientCreation(e) => Some(e),
            RequestError::RequestSend(e) => Some(e),
            RequestError::ResponseBodyRead(e) => Some(e),
            RequestError::ResponseParse(e, _body) => Some(e),
            RequestError::BadRequest(_)
            | RequestError::InternalServerError(_)
            | RequestError::UnexpectedStatus { .. } => None,
        }
    }
}

impl From<reqwest::header::InvalidHeaderValue> for RequestError {
    fn from(error: reqwest::header::InvalidHeaderValue) -> Self {
        RequestError::AuthHeaderCreation(error)
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_builder() {
            RequestError::ClientCreation(error)
        } else if error.is_request() {
            RequestError::RequestSend(error)
        } else {
            RequestError::ResponseBodyRead(error)
        }
    }
}

/// Arguments for making a request.
pub struct RequestArgs<'a> {
    /// User agent string.
    pub user_agent: &'a str,
    /// HTTP method.
    pub method: reqwest::Method,
    /// URL to send the request to.
    pub url: &'a str,
    /// Bearer token for authentication.
    pub token: &'a str,
    /// Optional request body.
    pub body: Option<&'a str>,
}

pub async fn make_request<T: DeserializeOwned>(args: RequestArgs<'_>) -> Result<T, RequestError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", args.token))?,
    );
    if args.body.is_some() {
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .user_agent(args.user_agent)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client
        .request(args.method, args.url)
        .body(args.body.unwrap_or_default().to_string())
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;
    println!("{}", body);

    match status.as_u16() {
        200..299 => {
            Ok(serde_json::from_str(&body).map_err(|e| RequestError::ResponseParse(e, body))?)
        }
        400 => {
            let error_response: ErrorResponse = serde_json::from_str(&body)
                .map_err(|e| RequestError::ResponseParse(e, body.clone()))?;
            Err(RequestError::BadRequest(error_response))
        }
        500 => {
            let error_response: ErrorResponse = serde_json::from_str(&body)
                .map_err(|e| RequestError::ResponseParse(e, body.clone()))?;
            Err(RequestError::InternalServerError(error_response))
        }
        _ => Err(RequestError::UnexpectedStatus { status, body }),
    }
}
