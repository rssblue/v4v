use crate::alby::helpers::{make_request, RequestArgs, RequestError};

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
