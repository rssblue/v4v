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
    let total_split: u128 = splits.iter().map(|&x| x as u128).sum();

    if splits.is_empty() {
        return vec![];
    }

    // Create a vector of (index, split) pairs and sort it by split in descending order
    let mut indexed_splits: Vec<(usize, u128)> = splits
        .iter()
        .enumerate()
        .map(|(i, &s)| (i, s as u128))
        .collect();
    indexed_splits.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    let mut sat_amounts: Vec<u64> = vec![0; num_recipients];
    let mut remaining_sats: u128 = total_sats as u128;

    // First, give one sat to as many recipients as possible, prioritizing higher splits
    for &(index, _) in indexed_splits.iter() {
        if remaining_sats == 0 {
            break;
        }
        sat_amounts[index] = 1;
        remaining_sats -= 1;
    }

    if remaining_sats > 0 {
        // Distribute remaining sats based on split ratios
        if total_split > 0 {
            for &(index, split) in indexed_splits.iter() {
                let share = (split * remaining_sats) / total_split;
                sat_amounts[index] += share as u64;
            }
        }

        // Distribute any leftover sats to recipients with the highest splits
        let distributed_sats: u128 = sat_amounts.iter().map(|&x| x as u128).sum();
        remaining_sats = total_sats as u128 - distributed_sats;

        if remaining_sats > 0 {
            for &(index, _) in indexed_splits.iter() {
                if remaining_sats == 0 {
                    break;
                }
                sat_amounts[index] += 1;
                remaining_sats -= 1;
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
    let remote_percentage = remote_percentage.min(100);
    let local_percentage = 100 - remote_percentage;

    // Calculate total splits
    let total_local: u64 = local_splits.iter().sum();
    let total_remote: u64 = remote_splits.iter().sum();

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
            .fold(0, |acc, &x| gcd(acc, x));
        let new_local_splits = local_splits
            .iter()
            .map(|x| if *x == 0 { 0 } else { x / gcd_value })
            .collect();
        let new_remote_splits = remote_splits
            .iter()
            .map(|x| if *x == 0 { 0 } else { x / gcd_value })
            .collect();
        return (new_local_splits, new_remote_splits);
    }

    // Scale splits without division
    let scaled_local: Vec<u64> = local_splits
        .iter()
        .map(|&split| split * local_percentage * total_remote)
        .collect();

    let scaled_remote: Vec<u64> = remote_splits
        .iter()
        .map(|&split| split * remote_percentage * total_local)
        .collect();

    // Combine all values to find the overall GCD
    let mut all_values = scaled_local.clone();
    all_values.extend(scaled_remote.iter());

    let gcd_value = all_values
        .iter()
        .filter(|&&x| x != 0)
        .fold(0, |acc, &x| gcd(acc, x));

    // Simplify the results using the GCD
    let final_local = scaled_local
        .into_iter()
        .map(|x| if x == 0 { 0 } else { x / gcd_value })
        .collect();
    let final_remote = scaled_remote
        .into_iter()
        .map(|x| if x == 0 { 0 } else { x / gcd_value })
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
