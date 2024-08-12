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
