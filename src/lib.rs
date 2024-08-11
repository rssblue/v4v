pub struct SplitRecipient {
    pub split: u64,
}

impl From<u64> for SplitRecipient {
    fn from(split: u64) -> Self {
        SplitRecipient { split }
    }
}

impl From<u32> for SplitRecipient {
    fn from(split: u32) -> Self {
        SplitRecipient {
            split: split as u64,
        }
    }
}

impl From<u16> for SplitRecipient {
    fn from(split: u16) -> Self {
        SplitRecipient {
            split: split as u64,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SatRecipient {
    pub num_sats: u64,
}

pub fn compute_sat_recipients(
    split_recipients: Vec<SplitRecipient>,
    total_sats: u64,
) -> Vec<SatRecipient> {
    let total_split: u64 = split_recipients.iter().map(|r| r.split).sum();

    let mut sat_recipients = Vec::new();
    for split_recipient in split_recipients {
        let num_sats = (split_recipient.split * total_sats) / total_split;
        sat_recipients.push(SatRecipient { num_sats });
    }

    let mut remaining_sats = total_sats - sat_recipients.iter().map(|r| r.num_sats).sum::<u64>();

    // Distribute remaining sats by adding 1 sat to each recipient until we run out of sats.
    // If a recipient already has at least one sat in the first iteration, skip them. But continue adding sats later.
    let mut index = 0;
    let mut first_iteration = true;

    while remaining_sats > 0 {
        if index >= sat_recipients.len() {
            index = 0;
            first_iteration = false;
        }

        if !first_iteration || sat_recipients[index].num_sats == 0 {
            sat_recipients[index].num_sats += 1;
            remaining_sats -= 1;
        }

        index += 1;
    }

    sat_recipients
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_compute_sat_recipients_simple() {
        let split_recipients = vec![SplitRecipient { split: 50 }, SplitRecipient { split: 50 }];

        let sat_recipients = compute_sat_recipients(split_recipients, 1000);

        assert_eq!(
            sat_recipients,
            vec![
                SatRecipient { num_sats: 500 },
                SatRecipient { num_sats: 500 },
            ]
        );
    }

    #[test]
    fn test_compute_sat_recipients_insufficient() {
        let split_recipients = vec![SplitRecipient { split: 50 }, SplitRecipient { split: 50 }];

        let sat_recipients = compute_sat_recipients(split_recipients, 1);

        assert_eq!(
            sat_recipients,
            vec![SatRecipient { num_sats: 1 }, SatRecipient { num_sats: 0 },]
        );
    }
}
