pub use crate::svix::webhooks::{HeaderMap, WebhookError as Error};

/// Verifies Alby webhook requests.
///
/// ## Example Axum usage
/// ```ignore
/// use axum::{
///     extract::Json,
///     routing::post,
///     Router,
///     http::StatusCode,
/// };
///
/// fn router() -> Router {
///     Router::new()
///         .route(
///             "/alby-webhook",
///             post(webhook_handler)
///         )
/// }
///
/// async fn webhook_handler(
///     headers: http::header::HeaderMap,
///     Json(body): Json<serde_json::Value>,
/// ) -> StatusCode {
///     match v4v::verify_signature(&secret, body.to_string().as_bytes(), &headers) {
///         Ok(()) => {}
///         Err(e) => {
///             log::error!("Failed to verify webhook: {:?}", e);
///             return StatusCode::BAD_REQUEST;
///         }
///     };
///
///     match save_to_db(&body).await {
///         Ok(_) => StatusCode::NO_CONTENT,
///         Err(e) => {
///             log::error!("Failed to record Alby payment: {}", e);
///             StatusCode::INTERNAL_SERVER_ERROR
///         }
///     }
/// }
/// ```
pub fn verify_signature<HM: HeaderMap>(
    secret: &str,
    payload: &[u8],
    headers: &HM,
) -> Result<(), Error> {
    crate::svix::webhooks::Webhook::new(secret)?.verify(payload, headers)
}
