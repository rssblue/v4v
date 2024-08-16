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
