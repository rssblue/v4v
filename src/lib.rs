#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod svix_webhooks;
pub use svix_webhooks::{HeaderMap, WebhookError};

/// Distributes [satoshis (sats)](https://en.wikipedia.org/wiki/Bitcoin#Units_and_divisibility) to
/// a list of recipients based on their splits.
///
/// Tries to ensure that every recipient get at least one sat. This makes it so that all the
/// recipients receive [TLV records](https://github.com/Podcastindex-org/podcast-namespace/blob/main/value/blip-0010.md)
/// associated with the payment.
///
/// If there aren't enough sats, the function will prioritize recipients with higher splits.
///
/// ## Example 1
/// ```rust
/// let splits = vec![60, 40];
/// let total_sats = 1000;
/// assert_eq!(v4v::compute_sat_recipients(&splits, total_sats), vec![600, 400]);
/// ```
///
/// ## Example 2
/// ```rust
/// let splits = vec![1, 99];
/// let total_sats = 10;
/// // It's ensured that the recipient with 1% split still gets at least 1 sat:
/// assert_eq!(v4v::compute_sat_recipients(&splits, total_sats), vec![1, 9]);
/// ```
///
/// ## Example 3
/// ```rust
/// let splits = vec![1, 99];
/// let total_sats = 1;
/// // There is only 1 sat available to distribute, so the recipient with the larger split gets it:
/// assert_eq!(v4v::compute_sat_recipients(&splits, total_sats), vec![0, 1]);
/// ```
pub fn compute_sat_recipients(splits: &[u64], total_sats: u64) -> Vec<u64> {
    let total_split: u64 = splits.iter().sum();

    let mut sat_amounts: Vec<u64> = splits
        .iter()
        .map(|&split| {
            if total_split == 0 {
                0
            } else {
                split * total_sats / total_split
            }
        })
        .collect();

    let distributed_sats: u64 = sat_amounts.iter().sum();
    let mut remaining_sats = total_sats - distributed_sats;

    if remaining_sats > 0 {
        // Create a vector of indices, sorted by split amount in descending order
        let mut indices: Vec<usize> = (0..splits.len()).collect();
        indices.sort_unstable_by(|&a, &b| splits[b].cmp(&splits[a]));

        // First pass: distribute to recipients with 0 sats, prioritizing higher splits
        for &i in &indices {
            if remaining_sats == 0 {
                break;
            }
            if sat_amounts[i] == 0 {
                sat_amounts[i] += 1;
                remaining_sats -= 1;
            }
        }

        // Second pass: distribute remaining sats to all recipients, still prioritizing higher splits
        while remaining_sats > 0 {
            for &i in &indices {
                if remaining_sats == 0 {
                    break;
                }
                sat_amounts[i] += 1;
                remaining_sats -= 1;
            }
        }
    }

    // Redundant check to make sure we are distributing the initial amount:
    if sat_amounts.iter().sum::<u64>() != total_sats {
        panic!("Distributed sats ({distributed_sats}) != total sats ({total_sats}) (splits = {splits:?}, sat_amounts = {sat_amounts:?})");
    }

    sat_amounts
}

/// Represents a share- or percentage-based recipient.
///
/// Percentage fees as part of the Podcasting 2.0 spec are
/// [controversial](https://github.com/Podcastindex-org/podcast-namespace/pull/596), but a hosting
/// company or an app may still find the concept useful. This enum is used to convert everything to
/// share-like splits, which are widely supported.
pub enum GenericRecipient {
    /// Share-based recipient.
    ShareBased {
        /// Number of shares.
        num_shares: u64,
    },
    /// Percentage-based recipient.
    PercentageBased {
        /// Percentage of the total.
        percentage: u64,
    },
}

/// Calculates the greatest common divisor of two numbers.
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Represents an error when converting a list of share- or percentage-based recipients into a list of share-like splits.
#[derive(PartialEq)]
pub enum RecipientsToSplitsError {
    /// The total fee exceeds 100%.
    TotalFeeExceeds100,
    /// The total fee is 100%, but there are non-fee recipients.
    FeeIs100ButNonFeeRecipientsExist,
}
impl std::fmt::Display for RecipientsToSplitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecipientsToSplitsError::TotalFeeExceeds100 => {
                write!(f, "Total fees exceeds 100%")
            }
            RecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist => {
                write!(f, "Total fees equal 100%, but non-fee recipients exist")
            }
        }
    }
}
impl std::fmt::Debug for RecipientsToSplitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Converts a list of [generic recipients](crate::GenericRecipient) into a list of share-like splits.
///
/// Share-based recipients maintain the same ratios between themselves after percentage-based
/// recipients are included.
///
/// ## Example 1
/// ```rust
/// let recipients = vec![
///     v4v::GenericRecipient::ShareBased { num_shares: 50 },
///     v4v::GenericRecipient::ShareBased { num_shares: 50 },
///     v4v::GenericRecipient::PercentageBased { percentage: 1 },
/// ];
/// // Share-based recipients still receive sats in the 50/50 ratio between them. But
/// // overall, they get 49.5% each, and the percentage-based recipient gets the required 1%.
/// // That's because 99/(99+99+2) = 49.5% and 2/(99+99+2) = 1%.
/// assert_eq!(v4v::fee_recipients_to_splits(&recipients), Ok(vec![99, 99, 2]));
pub fn fee_recipients_to_splits(
    recipients: &[GenericRecipient],
) -> Result<Vec<u64>, RecipientsToSplitsError> {
    let total_percentage: u64 = recipients
        .iter()
        .filter_map(|r| match r {
            GenericRecipient::PercentageBased { percentage } => Some(*percentage),
            _ => None,
        })
        .sum();

    if total_percentage > 100 {
        return Err(RecipientsToSplitsError::TotalFeeExceeds100);
    }

    let share_recipients: Vec<&GenericRecipient> = recipients
        .iter()
        .filter(|r| matches!(r, GenericRecipient::ShareBased { .. }))
        .collect();

    if total_percentage == 100 && !share_recipients.is_empty() {
        return Err(RecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist);
    }

    let remaining_percentage = 100 - total_percentage;
    let total_shares: u64 = share_recipients
        .iter()
        .filter_map(|r| match r {
            GenericRecipient::ShareBased { num_shares } => Some(*num_shares),
            _ => None,
        })
        .sum();

    let mut result = Vec::with_capacity(recipients.len());

    for recipient in recipients {
        match recipient {
            GenericRecipient::ShareBased { num_shares } => {
                result.push(num_shares * remaining_percentage);
            }
            GenericRecipient::PercentageBased { percentage } => {
                if share_recipients.is_empty() {
                    result.push(*percentage);
                } else {
                    result.push(*percentage * total_shares);
                }
            }
        }
    }

    // Find the GCD of all non-zero values to normalize the results
    let gcd_value = result
        .iter()
        .filter(|&&x| x != 0)
        .fold(0, |acc, &x| gcd(acc, x));
    if gcd_value > 1 {
        result = result
            .into_iter()
            .map(|x| if x == 0 { 0 } else { x / gcd_value })
            .collect();
    }

    Ok(result)
}

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
///     match v4v::verify_alby_signature(&secret, body.to_string().as_bytes(), &headers) {
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
pub fn verify_alby_signature<HM: HeaderMap>(
    secret: &str,
    payload: &[u8],
    headers: &HM,
) -> Result<(), WebhookError> {
    crate::svix_webhooks::Webhook::new(secret)?.verify(payload, headers)
}
