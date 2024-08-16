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

pub struct FeeRecipient {
    pub split: u64,
    /// When `fee` is `true`, the splits is a percentage fee taken off the top.
    pub fee: bool,
}
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

#[derive(Debug, PartialEq)]
pub enum FeeRecipientsToSplitsError {
    TotalFeeExceeds100,
    FeeIs100ButNonFeeRecipientsExist,
}

/// Converts a list of fee recipients into a list of share-like splits.
/// Non-fee recipients maintain the same ratios.
/// If fee splits exceed 100%, the function will return an error.
/// If fee splits are equal to 100% and there exist some non-fee recipients,
/// the function will return an error.
pub fn fee_recipients_to_splits(
    fee_recipients: Vec<FeeRecipient>,
) -> Result<Vec<u64>, FeeRecipientsToSplitsError> {
    let total_fee: u64 = fee_recipients
        .iter()
        .filter(|r| r.fee)
        .map(|r| r.split)
        .sum();

    if total_fee > 100 {
        return Err(FeeRecipientsToSplitsError::TotalFeeExceeds100);
    }

    let non_fee_recipients: Vec<&FeeRecipient> = fee_recipients.iter().filter(|r| !r.fee).collect();

    if total_fee == 100 && !non_fee_recipients.is_empty() {
        return Err(FeeRecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist);
    }

    let remaining_split = 100 - total_fee;
    let total_non_fee_split: u64 = non_fee_recipients.iter().map(|r| r.split).sum();

    let mut result = Vec::new();

    for recipient in &fee_recipients {
        if recipient.fee {
            result.push(recipient.split * 100); // Multiply by 100 to maintain precision
        } else {
            if total_non_fee_split == 0 {
                result.push(0);
            } else {
                let adjusted_split = recipient.split * remaining_split * 100 / total_non_fee_split;
                result.push(adjusted_split);
            }
        }
    }

    // Normalize the results
    let gcd_value = result.iter().fold(0, |acc, &x| gcd(acc, x));

    result = result.into_iter().map(|x| x / gcd_value).collect();

    Ok(result)
}
