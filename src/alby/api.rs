pub use crate::alby::helpers::RequestError;
use crate::alby::helpers::{make_request, ErrorResponse, RequestArgs};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
        /// Arbitrary text (included in the BOLT11 invoice).
        pub description: Option<String>,
        /// Name of payer (not included in the BOLT11 invoice)
        pub payer_name: Option<String>,
    }

    /// Request body for [create_invoice].
    #[derive(Debug, serde::Serialize)]
    struct CreateInvoiceRequestBody {
        /// The amount of sats.
        #[serde(rename = "amount")]
        pub num_sats: u64,
        /// Arbitary metadata.
        pub metadata: serde_json::Value,
        /// Arbitrary text (included in the BOLT11 invoice).
        pub description: Option<String>,
        /// Name of payer (not included in the BOLT11 invoice)
        pub payer_name: Option<String>,
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
        let request_body = CreateInvoiceRequestBody {
            num_sats: args.num_sats,
            metadata: args.metadata.clone(),
            description: args.description.clone(),
            payer_name: args.payer_name.clone(),
        };

        let body = serde_json::to_string(&request_body)
            .map_err(|e| RequestError::Unexpected(e.to_string()))?;

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

/// Alby API functions related to payments.
///
/// Read more at
/// <https://guides.getalby.com/developer-guide/v/alby-wallet-api/reference/api-reference/payments>.
pub mod payments {
    use super::*;

    /// Arguments for [keysend].
    pub struct KeysendArgs<'a> {
        /// User agent string.
        pub user_agent: &'a str,
        /// Bearer token for authentication.
        pub token: &'a str,
        /// The amount of sats.
        pub num_sats: u64,
        /// Destination node pubkey.
        pub dest_pubkey: &'a str,
        /// Custom records.
        pub custom_records: HashMap<String, String>,
    }

    /// Request body for [keysend].
    #[derive(Debug, serde::Serialize)]
    struct KeysendRequest {
        /// The amount of sats.
        #[serde(rename = "amount")]
        pub num_sats: u64,
        /// Destination node pubkey.
        #[serde(rename = "destination")]
        pub dest_pubkey: String,
        /// Custom records.
        pub custom_records: HashMap<String, String>,
    }

    /// Response for a successful keysend.
    #[derive(Debug, serde::Deserialize)]
    pub struct KeysendResponse {
        /// The amount of sats.
        #[serde(rename = "amount")]
        pub num_sats: u64,
        /// Description.
        pub description: String,
        /// Description hash.
        pub description_hash: String,
        /// Destination node pubkey.
        #[serde(rename = "destination")]
        pub dest_pubkey: String,
        /// Fee in sats.
        #[serde(rename = "fee")]
        pub fee_in_sats: u64,
        /// Custom records.
        pub custom_records: HashMap<String, String>,
        /// Payment hash.
        pub payment_hash: String,
        /// Payment preimage.
        pub payment_preimage: String,
    }

    /// Send a keysend payment using the Alby API.
    pub async fn keysend(args: KeysendArgs<'_>) -> Result<KeysendResponse, RequestError> {
        let request_body = KeysendRequest {
            num_sats: args.num_sats,
            dest_pubkey: args.dest_pubkey.to_string(),
            custom_records: args.custom_records.clone(),
        };

        let body = serde_json::to_string(&request_body)
            .map_err(|e| RequestError::Unexpected(e.to_string()))?;

        let request_args = RequestArgs {
            user_agent: args.user_agent,
            method: reqwest::Method::POST,
            url: "https://api.getalby.com/payments/keysend",
            token: args.token,
            body: Some(&body),
        };

        make_request(request_args).await
    }

    /// Request body for [multi_keysend].
    #[derive(Debug, serde::Serialize)]
    struct MultiKeysendRequest {
        /// Array of keysend objects.
        pub keysends: Vec<KeysendRequest>,
    }

    /// Keysend item response for [multi_keysend].
    #[derive(Debug, serde::Deserialize)]
    pub struct MultiKeysendItemResponse {
        /// Error.
        pub error: ErrorResponse,
        /// Keysend response.
        pub keysend: KeysendResponse,
    }

    /// Response for a successful [multi_keysend].
    #[derive(Debug, serde::Deserialize)]
    pub struct MultiKeysendResponse {
        /// Array of keysend responses.
        pub keysends: Vec<MultiKeysendItemResponse>,
    }

    /// [multi_keysend] keysend item.
    pub struct MultiKeysendItemArgs<'a> {
        /// The amount of sats.
        pub num_sats: u64,
        /// Destination node pubkey.
        pub dest_pubkey: &'a str,
        /// Custom records.
        pub custom_records: HashMap<String, String>,
    }

    /// Arguments for [keysend].
    pub struct MultiKeysendArgs<'a> {
        /// User agent string.
        pub user_agent: &'a str,
        /// Bearer token for authentication.
        pub token: &'a str,
        /// Keysend items.
        pub keysends: Vec<MultiKeysendItemArgs<'a>>,
    }

    /// Send multiple keysend payments using the Alby API.
    pub async fn multi_keysend(
        args: MultiKeysendArgs<'_>,
    ) -> Result<MultiKeysendResponse, RequestError> {
        let mut keysends = Vec::new();

        for item in args.keysends.iter() {
            let request = KeysendRequest {
                num_sats: item.num_sats,
                dest_pubkey: item.dest_pubkey.to_string(),
                custom_records: item.custom_records.clone(),
            };

            keysends.push(request);
        }

        let request_body = MultiKeysendRequest { keysends };

        let body = serde_json::to_string(&request_body)
            .map_err(|e| RequestError::Unexpected(e.to_string()))?;

        let request_args = RequestArgs {
            user_agent: args.user_agent,
            method: reqwest::Method::POST,
            url: "https://api.getalby.com/payments/multi",
            token: args.token,
            body: Some(&body),
        };

        make_request(request_args).await
    }
}
