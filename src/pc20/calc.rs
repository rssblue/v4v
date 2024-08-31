use std::cmp::Ordering;

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
/// assert_eq!(v4v::pc20::calc::compute_sat_recipients(&splits, total_sats), vec![600, 400]);
/// ```
///
/// ## Example 2
/// ```rust
/// let splits = vec![1, 99];
/// let total_sats = 10;
/// // It's ensured that the recipient with 1% split still gets at least 1 sat:
/// assert_eq!(v4v::pc20::calc::compute_sat_recipients(&splits, total_sats), vec![1, 9]);
/// ```
///
/// ## Example 3
/// ```rust
/// let splits = vec![1, 99];
/// let total_sats = 1;
/// // There is only 1 sat available to distribute, so the recipient with the larger split gets it:
/// assert_eq!(v4v::pc20::calc::compute_sat_recipients(&splits, total_sats), vec![0, 1]);
/// ```
pub fn compute_sat_recipients(splits: &[u64], total_sats: u64) -> Vec<u64> {
    let num_recipients = splits.len();

    if splits.is_empty() || total_sats == 0 {
        return vec![0; num_recipients];
    }

    // Use u128 for calculations to avoid overflow
    let total_split: u128 = splits.iter().map(|&x| x as u128).sum();

    if total_split == 0 {
        // If all splits are zero, distribute evenly
        let base_amount = total_sats / num_recipients as u64;
        let remainder = total_sats % num_recipients as u64;
        let mut result = vec![base_amount; num_recipients];
        for result_item in result.iter_mut().take(remainder as usize) {
            *result_item += 1;
        }
        return result;
    }

    // Distribute sats proportionally
    let mut sat_amounts: Vec<u64> = splits
        .iter()
        .map(|&split| ((split as u128 * total_sats as u128) / total_split) as u64)
        .collect();

    // Give one sat to everyone who doesn't have it:
    for amount in sat_amounts.iter_mut() {
        if *amount == 0 {
            *amount = 1;
        }
    }

    let mut balance: i64 = total_sats as i64 - sat_amounts.iter().sum::<u64>() as i64;

    // Create a vector of (index, split) pairs
    let mut indexed_splits: Vec<(usize, u128)> = splits
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, s)| (i, s as u128))
        .collect();

    // Using cmp
    match balance.cmp(&0) {
        Ordering::Less => {
            // Sort by increasing splits, then by decreasing index.
            indexed_splits.sort_by(|&(i1, s1), &(i2, s2)| s1.cmp(&s2).then(i2.cmp(&i1)));

            let initial_balance = balance;

            // First iteration: Remove from those that have at least two sats.
            for &(index, split) in &indexed_splits {
                // Remove amount proportional to the split.
                let amount_to_remove =
                    (split * initial_balance.unsigned_abs() as u128 / total_split).max(1);
                if balance < 0 && sat_amounts[index] > amount_to_remove as u64 {
                    sat_amounts[index] -= amount_to_remove as u64;
                    balance += amount_to_remove as i64;
                }
                if balance == 0 {
                    break;
                }
            }

            if balance < 0 {
                let mut indexed_splits_with_amounts: Vec<(usize, u128, u64)> = indexed_splits
                    .iter()
                    .map(|&(index, split)| (index, split, sat_amounts[index]))
                    .collect();

                // Remove in the following order:
                // 1) increasing split for recipients with at least two sats
                // 2) increasing split for recipients with at least one sat
                // 3) decreasing index
                indexed_splits_with_amounts.sort_by(|&(i1, s1, a1), &(i2, s2, a2)| {
                    match (a1 >= 2, a2 >= 2) {
                        (true, true) => s1.cmp(&s2),
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        (false, false) => match (a1 >= 1, a2 >= 1) {
                            (true, true) => s1.cmp(&s2),
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            (false, false) => i2.cmp(&i1),
                        },
                    }
                });

                // Update indexed_splits with the new order
                indexed_splits = indexed_splits_with_amounts
                    .into_iter()
                    .map(|(i, s, _)| (i, s))
                    .collect();

                for &(index, _) in &indexed_splits {
                    if balance < 0 && sat_amounts[index] > 0 {
                        sat_amounts[index] -= 1;
                        balance += 1;
                    }
                    if balance == 0 {
                        break;
                    }
                }
            }
        }
        Ordering::Equal => {}
        Ordering::Greater => {
            // Sort by decreasing splits for positive balance
            indexed_splits.sort_by_key(|&(_, split)| std::cmp::Reverse(split));

            // Single iteration: add sats to recipients
            for &(index, _) in &indexed_splits {
                if balance > 0 {
                    sat_amounts[index] += 1;
                    balance -= 1;
                } else {
                    break;
                }
            }
        }
    }

    sat_amounts
}

/// Similar to [compute_sat_recipients] but allows to use it with any type that uses splits.
pub fn compute_sat_recipients_generic<T: HasSplit + Clone>(
    values: &[T],
    total_sats: u64,
) -> Vec<u64> {
    let splits: Vec<u64> = values.iter().map(|v| v.get_split()).collect();
    compute_sat_recipients(&splits, total_sats)
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
fn gcd(a: u128, b: u128) -> u128 {
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

/// Converts a list of [generic recipients](GenericRecipient) into a list of share-like splits.
///
/// Share-based recipients maintain the same ratios between themselves after percentage-based
/// recipients are included.
///
/// ## Example 1
/// ```rust
/// let recipients = vec![
///     v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
///     v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
///     v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
/// ];
/// // Share-based recipients still receive sats in the 50/50 ratio between them. But
/// // overall, they get 49.5% each, and the percentage-based recipient gets the required 1%.
/// // That's because 99/(99+99+2) = 49.5% and 2/(99+99+2) = 1%.
/// assert_eq!(v4v::pc20::calc::fee_recipients_to_splits(&recipients), Ok(vec![99, 99, 2]));
pub fn fee_recipients_to_splits(
    recipients: &[GenericRecipient],
) -> Result<Vec<u64>, RecipientsToSplitsError> {
    let total_percentage: u128 = recipients
        .iter()
        .filter_map(|r| match r {
            GenericRecipient::PercentageBased { percentage } => Some(*percentage as u128),
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

    let remaining_percentage: u128 = 100 - total_percentage;
    let total_shares: u128 = share_recipients
        .iter()
        .filter_map(|r| match r {
            GenericRecipient::ShareBased { num_shares } => Some(*num_shares as u128),
            _ => None,
        })
        .sum();

    let mut result = Vec::with_capacity(recipients.len());

    for recipient in recipients {
        match recipient {
            GenericRecipient::ShareBased { num_shares } => {
                let value = (*num_shares as u128) * remaining_percentage;
                result.push(value);
            }
            GenericRecipient::PercentageBased { percentage } => {
                let value = if share_recipients.is_empty() {
                    *percentage as u128
                } else {
                    (*percentage as u128) * total_shares
                };
                result.push(value);
            }
        }
    }

    // Find the GCD of all non-zero values to normalize the results
    let gcd_value = result
        .iter()
        .filter(|&&x| x != 0)
        .fold(0u128, |acc, &x| gcd(acc, x));

    if gcd_value > 1 {
        result = result
            .into_iter()
            .map(|x| if x == 0 { 0 } else { x / gcd_value })
            .collect();
    }

    // Convert result back to u64, scaling all values proportionally if any overflow occurs
    let max_value = *result.iter().max().unwrap_or(&0);
    if max_value > u64::MAX as u128 {
        let scale_factor = (u64::MAX as f64) / (max_value as f64);
        Ok(result
            .into_iter()
            .map(|x| (x as f64 * scale_factor).round() as u64)
            .collect())
    } else {
        Ok(result.into_iter().map(|x| x as u64).collect())
    }
}

/// Similar to [fee_recipients_to_splits] but allows to use it with any type that uses splits and
/// implements `Into<crate::GenericRecipient>`.
pub fn fee_recipients_to_splits_generic<T: Into<GenericRecipient> + HasSplit + Clone>(
    recipients: &[T],
) -> Result<Vec<T>, RecipientsToSplitsError> {
    let generic_recipients: Vec<GenericRecipient> =
        recipients.iter().map(|r| (*r).clone().into()).collect();

    let splits = fee_recipients_to_splits(&generic_recipients)?;

    let mut result = Vec::with_capacity(recipients.len());
    for (recipient, &split) in recipients.iter().zip(splits.iter()) {
        let mut recipient = (*recipient).clone();
        recipient.set_split(split);
        result.push(recipient);
    }

    Ok(result)
}

/// Scales the splits such that `remote_splits` would constitute `remote_percentage` of the total,
/// and `local_splits` would constitute the rest.
///
/// ## Example
/// ```rust
/// let local_splits = vec![50, 50];
/// let remote_splits = vec![1];
/// let remote_percentage = 90;
/// // The remote split is 18, because 18/(18+1+1) = 90%. The local splits are 1 and 1, because
/// // 1/(18+1+1) = 5%.
/// assert_eq!(v4v::pc20::calc::use_remote_splits(&local_splits, &remote_splits, remote_percentage), (vec![1, 1], vec![18]));
pub fn use_remote_splits(
    local_splits: &[u64],
    remote_splits: &[u64],
    remote_percentage: u64,
) -> (Vec<u64>, Vec<u64>) {
    // Cap remote_percentage at 100
    let remote_percentage = remote_percentage.min(100) as u128;
    let local_percentage = 100u128 - remote_percentage;

    // Calculate total splits
    let total_local: u128 = local_splits.iter().map(|&x| x as u128).sum();
    let total_remote: u128 = remote_splits.iter().map(|&x| x as u128).sum();

    // If either total is 0, we need to handle this specially
    if total_local == 0 || total_remote == 0 {
        let all_values = local_splits
            .iter()
            .chain(remote_splits.iter())
            .copied()
            .collect::<Vec<_>>();
        let gcd_value = all_values
            .iter()
            .filter(|&&x| x != 0)
            .fold(0u128, |acc, &x| gcd(acc, x as u128));
        let new_local_splits = local_splits
            .iter()
            .map(|&x| {
                if x == 0 {
                    0
                } else {
                    (x as u128 / gcd_value) as u64
                }
            })
            .collect();
        let new_remote_splits = remote_splits
            .iter()
            .map(|&x| {
                if x == 0 {
                    0
                } else {
                    (x as u128 / gcd_value) as u64
                }
            })
            .collect();
        return (new_local_splits, new_remote_splits);
    }

    // Scale splits without division
    let scaled_local: Vec<u128> = local_splits
        .iter()
        .map(|&split| (split as u128) * local_percentage * total_remote)
        .collect();

    let scaled_remote: Vec<u128> = remote_splits
        .iter()
        .map(|&split| (split as u128) * remote_percentage * total_local)
        .collect();

    // Combine all values to find the overall GCD
    let mut all_values = scaled_local.clone();
    all_values.extend(scaled_remote.iter().cloned());

    let gcd_value = all_values
        .iter()
        .filter(|&&x| x != 0)
        .fold(0u128, |acc, &x| gcd(acc, x));

    // Simplify the results using the GCD
    let final_local: Vec<u64> = scaled_local
        .into_iter()
        .map(|x| if x == 0 { 0 } else { (x / gcd_value) as u64 })
        .collect();
    let final_remote: Vec<u64> = scaled_remote
        .into_iter()
        .map(|x| if x == 0 { 0 } else { (x / gcd_value) as u64 })
        .collect();

    (final_local, final_remote)
}

/// Trait for types that have a split.
pub trait HasSplit {
    /// Set split.
    fn set_split(&mut self, split: u64);
    /// Get split.
    fn get_split(&self) -> u64;
}

/// Similar to [use_remote_splits] but allows to use it with any type that uses splits.
pub fn use_remote_splits_generic<T: HasSplit + Clone>(
    local_values: &[T],
    remote_values: &[T],
    remote_percentage: u64,
) -> Vec<T> {
    let local_splits: Vec<u64> = local_values.iter().map(|v| v.get_split()).collect();
    let remote_splits: Vec<u64> = remote_values.iter().map(|v| v.get_split()).collect();
    let (new_local_splits, new_remote_splits) =
        use_remote_splits(&local_splits, &remote_splits, remote_percentage);

    let mut result = Vec::with_capacity(local_values.len() + remote_values.len());
    for (value, &split) in local_values.iter().zip(new_local_splits.iter()) {
        let mut value = value.clone();
        value.set_split(split);
        result.push(value);
    }
    for (value, &split) in remote_values.iter().zip(new_remote_splits.iter()) {
        let mut value = value.clone();
        value.set_split(split);
        result.push(value);
    }

    result
}
