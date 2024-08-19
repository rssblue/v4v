use serde_json::Value;

use crate::pc20::tlv::Record;
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

/// Alby invoice obtained via webhook request.
#[derive(Debug, serde::Deserialize)]
pub struct AlbyInvoice {
    /// 24 alphanumeric characters
    pub identifier: String,

    /// e.g., "incoming" or "outgoing"
    #[serde(rename = "type")]
    pub type_: String,

    /// Description.
    #[serde(default)]
    pub memo: Option<String>,

    /// State of the invoice, e.g., "SETTLED">
    pub state: String,

    /// Arbitrary data added during the invoice creation.
    #[serde(default)]
    pub metadata: Value,

    /// Payer name.
    #[serde(default)]
    pub payer_name: Option<String>,

    /// Amount in sats.
    #[serde(rename = "amount")]
    pub num_sats: u64,

    /// When the invoice was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// bLIP-10 TLV record.
    #[serde(
        default,
        deserialize_with = "crate::pc20::tlv::deserialize_untrusted_tlv_record"
    )]
    pub boostagram: Option<Record>,
}

/// Extracts an Alby invoice from a webhook request body.
pub fn extract_alby_invoice(body: &Value) -> Result<AlbyInvoice, String> {
    serde_json::from_value(body.clone()).map_err(|e| e.to_string())
}
