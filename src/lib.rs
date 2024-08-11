pub fn compute_sat_recipients(splits: Vec<u64>, total_sats: u64) -> Vec<u64> {
    let total_split: u64 = splits.iter().sum();

    let mut sat_amounts = Vec::new();
    for split in splits {
        let num_sats = if total_split == 0 {
            0
        } else {
            split * total_sats / total_split
        };
        sat_amounts.push(num_sats);
    }

    let distributed_sats: u64 = sat_amounts.iter().sum();
    let mut remaining_sats = total_sats - distributed_sats;

    // Distribute remaining sats by adding 1 sat to each recipient until we run out of sats.
    // If a recipient already has at least one sat, skip them in this first iteration.
    for sat in sat_amounts.iter_mut() {
        if remaining_sats == 0 {
            break;
        }

        if *sat == 0 {
            *sat += 1;
            remaining_sats -= 1;
        }
    }
    // If we still have sats left, distribute them to all recipients.
    if remaining_sats > 0 {
        for sat in sat_amounts.iter_mut() {
            if remaining_sats == 0 {
                break;
            }

            *sat += 1;
            remaining_sats -= 1;
        }
    }

    sat_amounts
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_compute_sat_recipients_simple() {
        let split_recipients = vec![50, 50];

        let sat_recipients = compute_sat_recipients(split_recipients, 1000);

        assert_eq!(sat_recipients, vec![500, 500,]);
    }

    #[test]
    fn test_compute_sat_recipients_insufficient() {
        let split_recipients = vec![50, 50];

        let sat_recipients = compute_sat_recipients(split_recipients, 1);

        assert_eq!(sat_recipients, vec![1, 0],);
    }

    #[test]
    fn test_compute_sat_recipients_insufficient_2() {
        let split_recipients = vec![0, 0];

        let sat_recipients = compute_sat_recipients(split_recipients, 1);

        assert_eq!(sat_recipients, vec![1, 0],);
    }

    #[test]
    fn test_compute_sat_recipients_insufficient_3() {
        let split_recipients = vec![50, 50, 1];

        let sat_recipients = compute_sat_recipients(split_recipients, 100);

        assert_eq!(sat_recipients, vec![50, 49, 1],);
    }
}
