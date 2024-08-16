#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Distributes [satoshis (sats)](https://en.wikipedia.org/wiki/Bitcoin#Units_and_divisibility) to
/// a list of recipients based on their splits.
///
/// Tries to ensure that every recipient get at least one sat. This makes it so that all the
/// recipients receive [TLV records](https://github.com/Podcastindex-org/podcast-namespace/blob/main/value/blip-0010.md)
/// associated with the payment.
///
/// If there aren't enough sats, the function will prioritize recipients with higher splits.
pub fn compute_sat_recipients(splits: Vec<u64>, total_sats: u64) -> Vec<u64> {
    let total_split: u64 = splits.iter().sum();

    let mut sat_amounts = Vec::new();
    for split in &splits {
        let num_sats = if total_split == 0 {
            0
        } else {
            split * total_sats / total_split
        };
        sat_amounts.push(num_sats);
    }

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
/// Non-fee recipients maintain the same ratios.
pub fn fee_recipients_to_splits(
    recipients: Vec<GenericRecipient>,
) -> Result<Vec<u64>, RecipientsToSplitsError> {
    let total_percentage: u64 = recipients
        .iter()
        .map(|r| match r {
            GenericRecipient::ShareBased { .. } => 0,
            GenericRecipient::PercentageBased { percentage } => *percentage,
        })
        .sum();

    if total_percentage > 100 {
        return Err(RecipientsToSplitsError::TotalFeeExceeds100);
    }

    let share_recipients: Vec<&GenericRecipient> = recipients
        .iter()
        .filter(|r| match r {
            GenericRecipient::ShareBased { .. } => true,
            GenericRecipient::PercentageBased { .. } => false,
        })
        .collect();

    if total_percentage == 100 && !share_recipients.is_empty() {
        return Err(RecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist);
    }

    let remaining_percentage = 100 - total_percentage;
    let total_shares: u64 = share_recipients
        .iter()
        .map(|r| match r {
            GenericRecipient::ShareBased { num_shares } => *num_shares,
            GenericRecipient::PercentageBased { .. } => 0,
        })
        .sum();

    let mut result = Vec::new();

    for recipient in &recipients {
        match recipient {
            GenericRecipient::ShareBased { num_shares } => {
                if total_shares == 0 {
                    result.push(0);
                } else {
                    result.push(*num_shares * remaining_percentage * 100 / total_shares);
                }
            }
            GenericRecipient::PercentageBased { percentage } => {
                result.push(*percentage * 100);
            }
        }
    }

    // Normalize the results
    let gcd_value = result.iter().fold(0, |acc, &x| gcd(acc, x));

    result = result.into_iter().map(|x| x / gcd_value).collect();

    Ok(result)
}
