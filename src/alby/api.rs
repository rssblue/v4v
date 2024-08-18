use crate::alby::helpers::{make_request, RequestArgs, RequestError};
use chrono::{DateTime, Utc};

/// Alby API functions related to the account.
///
/// Read more at
/// <https://guides.getalby.com/developer-guide/v/alby-wallet-api/reference/api-reference/account>.
pub mod account {
    use super::*;

    /// Response body for a successful [get_balance] request.
    #[derive(Debug, serde::Deserialize)]
    pub struct GetBalanceResponse {
        /// The balance amount.
        pub balance: u64,
        /// The currency of the balance.
        pub currency: String,
        /// The unit of the balance (e.g., "sat" for satoshis).
        pub unit: String,
    }

    /// Arguments for [get_balance].
    pub struct GetBalanceArgs<'a> {
        /// User agent string.
        pub user_agent: &'a str,
        /// Bearer token for authentication.
        pub token: &'a str,
    }

    /// Get Alby account balance.
    pub async fn get_balance(args: GetBalanceArgs<'_>) -> Result<GetBalanceResponse, RequestError> {
        let request_args = RequestArgs {
            user_agent: args.user_agent,
            method: reqwest::Method::GET,
            url: "https://api.getalby.com/balance",
            token: args.token,
            body: None,
        };

        make_request(request_args).await
    }
}

/// Alby API functions related to invoices.
///
/// Read more at
/// <https://guides.getalby.com/developer-guide/v/alby-wallet-api/reference/api-reference/invoices>.
pub mod invoices {
    use super::*;

    /// Arguments for [create_invoice].
    pub struct CreateInvoiceArgs<'a> {
        /// User agent string.
        pub user_agent: &'a str,
        /// Bearer token for authentication.
        pub token: &'a str,
        /// The amount of sats.
        pub num_sats: u64,
        /// Arbitary metadata.
        pub metadata: serde_json::Value,
    }

    /// Response for a successful invoice creation.
    #[derive(Debug, serde::Deserialize)]
    pub struct CreateInvoiceResponse {
        /// The expiration time of the invoice.
        pub expires_at: DateTime<Utc>,
        /// The payment hash of the invoice.
        pub payment_hash: String,
        /// The BOLT11 payment request string.
        pub payment_request: String,
    }

    /// Create an invoice using the Alby API.
    pub async fn create_invoice(
        args: CreateInvoiceArgs<'_>,
    ) -> Result<CreateInvoiceResponse, RequestError> {
        let request_body = serde_json::json!({
            "amount": args.num_sats,
            "metadata": args.metadata,
        });

        let body = serde_json::to_string(&request_body).map_err(RequestError::ResponseParse)?;

        let request_args = RequestArgs {
            user_agent: args.user_agent,
            method: reqwest::Method::POST,
            url: "https://api.getalby.com/invoices",
            token: args.token,
            body: Some(&body),
        };

        make_request(request_args).await
    }
}
