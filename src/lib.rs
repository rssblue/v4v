#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// [Alby](https://getalby.com)-specific types and functions.
pub mod alby {
    /// Alby API types and functions.
    pub mod api;
    /// Helper functions.
    mod helpers;
    /// Extra Alby types.
    pub mod types;
    /// Alby webhook utilities.
    pub mod webhooks;
}

/// This is forked from <https://github.com/svix/svix-webhooks/blob/main/rust/src/webhooks.rs> to
/// minimize the amount of dependencies.
mod svix {
    pub mod webhooks;
}

/// Podcasting-related helpers.
pub mod pc20 {
    /// Functions related to sat caculations.
    pub mod calc;
    /// Utilities enabling to forward BOLT11 payments to keysend recipients.
    ///
    /// ## Example usage
    /// ```ignore
    /// use axum::{
    ///     extract::Json,
    ///     routing::post,
    ///     Router,
    ///     http::StatusCode,
    /// };
    ///
    /// const ALBY_TOKEN: &str = "my_secret_alby_token";
    /// const USER_AGENT: &str = "MyApp/1.0";
    ///
    /// fn router() -> Router {
    ///     Router::new()
    ///         .route(
    ///             "/generate-forwarding-invoice",
    ///             post(generate_forwarding_invoice)
    ///         )
    ///         .route(
    ///             "/alby-incoming-payments",
    ///             post(webhook_handler)
    ///         )
    /// }
    ///
    /// #[derive(serde::Deserialize)]
    /// struct GenerateForwardingInvoiceRequest {
    ///     pub payment_info: v4v::pc20::payments::PaymentInfo,
    ///     pub recipients: Vec<v4v::pc20::payments::PaymentRecipientInfo>,
    /// }
    ///
    /// #[derive(serde::Serialize)]
    /// struct GenerateForwardingInvoiceResponse {
    ///     pub invoice: String,
    /// }
    ///
    /// async fn generate_forwarding_invoice(
    ///     Json(body): Json<GenerateForwardingInvoiceRequest>,
    /// ) -> Result<Json<GenerateForwardingInvoiceResponse>, StatusCode> {
    ///     // Might also want to do additional checks before generating invoice, e.g., limiting
    ///     // the amount.
    ///
    ///     let resp = match v4v::pc20::forwarding::create_invoice(
    ///         v4v::pc20::forwarding::CreateInvoiceArgs {
    ///             user_agent: USER_AGENT,
    ///             token: ALBY_TOKEN,
    ///             payment_info: body.payment_info,
    ///             recipients: body.recipients,
    ///         }).await {
    ///             Ok(resp) => resp,
    ///             Err(e) => {
    ///                 log::error!("Failed to create invoice: {}", e);
    ///                 return Err(StatusCode::INTERNAL_SERVER_ERROR);
    ///             },
    ///         };
    ///
    ///     Ok(Json(GenerateForwardingInvoiceResponse {
    ///         invoice: resp.payment_request,
    ///     }))
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
    ///     let alby_invoice = v4v::alby::webhooks::extract_alby_invoice(&body)?;
    ///     if alby_invoice.status != "SETTLED".to_string() {
    ///         return StatusCode::OK;
    ///     }
    ///
    ///     let metadata = match v4v::pc20::forwarding::CreateInvoiceMetadata::try_from(
    ///         alby_invoice.metadata,
    ///         ) {
    ///             Ok(metadata) => metadata,
    ///             Err(e) => return StatusCode::OK,
    ///         };
    ///
    ///     let payment_info = metadata.forwarding_data.payment_info;
    ///     let recipients = metadata.forwarding_data.recipients;
    ///     // Trim if the sum of payments somehow exceeds the total amount received.
    ///     let recipients = v4v::pc20::forwarding::clip_recipients_at_amount(alby_invoice.num_sats, &recipients);
    ///
    ///     match v4v::pc20::forwarding::forward_payments(v4v::pc20::forwarding::ForwardPaymentArgs {
    ///         user_agent: USER_AGENT,
    ///         token: ALBY_TOKEN,
    ///         payment_info,
    ///         recipients,
    ///         }).await {
    ///             Ok(_) => StatusCode::NO_CONTENT,
    ///             Err(e) => {
    ///                 log::error!("Failed to forward payments: {}", e);
    ///                 StatusCode::INTERNAL_SERVER_ERROR
    ///             },
    ///         }
    /// }
    /// ```
    pub mod forwarding;
    /// Podcasting-related payment utilities.
    pub mod payments;
    /// Utilities related to Podcasting 2.0 TLV records.
    pub mod tlv;
}
