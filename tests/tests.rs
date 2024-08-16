use pretty_assertions::assert_eq;

macro_rules! compute_sat_recipients_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<compute_sat_recipients_ $name>]() {
                    struct TestCase {
                        splits: Vec<u64>,
                        total_sats: u64,
                        expected_sats: Vec<u64>,
                    }
                    assert_eq!(v4v::compute_sat_recipients($value.splits, $value.total_sats), $value.expected_sats);
                }
            )*
        }
    }
}

compute_sat_recipients_tests! {
    case_1: TestCase {
        splits: vec![50, 50],
        total_sats: 1000,
        expected_sats: vec![500, 500],
    },
    case_2: TestCase {
        splits: vec![50, 50],
        total_sats: 1,
        expected_sats: vec![1, 0],
    },
    case_3: TestCase {
        // Some hosts/programs (like the Split Kit) might generate 0-valued splits, especially for live wallet switching.
        splits: vec![0, 0],
        total_sats: 1,
        expected_sats: vec![1, 0],
    },
    case_4: TestCase {
        splits: vec![50, 50, 1],
        total_sats: 1,
        expected_sats: vec![1, 0, 0],
    },
    case_5: TestCase {
        splits: vec![50, 50, 1],
        total_sats: 100,
        expected_sats: vec![50, 49, 1],
    },
    case_6: TestCase {
        // If not all recipients can receive sats, ones with higher splits should be prioritized.
        splits: vec![1, 50, 50],
        total_sats: 1,
        expected_sats: vec![0, 1, 0],
    },
}

macro_rules! fee_recipients_to_splits_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<fee_recipients_to_splits_ $name>]() {
                    struct TestCase {
                        recipients: Vec<v4v::GenericRecipient>,
                        expected_splits: Result<Vec<u64>, v4v::RecipientsToSplitsError>,
                    }
                    assert_eq!(v4v::fee_recipients_to_splits($value.recipients), $value.expected_splits);
                }
            )*
        }
    }
}

fee_recipients_to_splits_tests! {
    case_1: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
        ],
        expected_splits: Ok(vec![1, 1]),
    },
    case_2: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::PercentageBased { percentage: 2 },
        ],
        expected_splits: Ok(vec![49, 49, 2]),
    },
    case_3: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![99, 99, 2]),
    },
    case_4: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![49, 49, 1, 1]),
    },
    case_5: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::GenericRecipient::PercentageBased { percentage: 2 },
        ],
        expected_splits: Ok(vec![49, 49, 2]),
    },
    case_6: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 100 },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_7: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 2 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![2, 1]),
    },
    case_8: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::GenericRecipient::ShareBased { num_shares: 40 },
        ],
        expected_splits: Ok(vec![1, 2, 3, 4]),
    },
    case_9: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::GenericRecipient::ShareBased { num_shares: 40 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![99, 198, 297, 396, 10]),
    },
    case_10: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::GenericRecipient::ShareBased { num_shares: 40 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![49, 98, 147, 196, 5, 5]),
    },
    // Change order:
    case_11: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::GenericRecipient::ShareBased { num_shares: 40 },
        ],
        expected_splits: Ok(vec![5, 49, 98, 147, 5, 196]),
    },
    case_12: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 100 },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_13: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_14: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 101 },
        ],
        expected_splits: Err(v4v::RecipientsToSplitsError::TotalFeeExceeds100),
    },
    case_15: TestCase {
        recipients: vec![
            v4v::GenericRecipient::PercentageBased { percentage: 100 },
            v4v::GenericRecipient::ShareBased { num_shares: 1 },
        ],
        expected_splits: Err(v4v::RecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist),
    },
    case_16: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::GenericRecipient::ShareBased { num_shares: 40 },
            v4v::GenericRecipient::ShareBased { num_shares: 3 },
            v4v::GenericRecipient::ShareBased { num_shares: 2 },
            v4v::GenericRecipient::ShareBased { num_shares: 2 },
            v4v::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::GenericRecipient::PercentageBased { percentage: 2 },
        ],
        expected_splits: Ok(vec![50, 40, 3, 2, 2, 1, 2]),
    },
    case_17: TestCase {
        recipients: vec![
            v4v::GenericRecipient::ShareBased { num_shares: 99999 },
            v4v::GenericRecipient::ShareBased { num_shares: 1 },
        ],
        expected_splits: Ok(vec![99999, 1]),
    },
}
