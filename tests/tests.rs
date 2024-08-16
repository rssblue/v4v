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
                        recipients: Vec<v4v::FeeRecipient>,
                        expected_splits: Result<Vec<u64>, v4v::FeeRecipientsToSplitsError>,
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
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 50, fee: false },
        ],
        expected_splits: Ok(vec![1, 1]),
    },
    case_2: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 2, fee: true },
        ],
        expected_splits: Ok(vec![49, 49, 2]),
    },
    case_3: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 1, fee: true },
        ],
        expected_splits: Ok(vec![99, 99, 2]),
    },
    case_4: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 1, fee: true },
            v4v::FeeRecipient { split: 1, fee: true },
        ],
        expected_splits: Ok(vec![49, 49, 1, 1]),
    },
    case_5: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 1, fee: false },
            v4v::FeeRecipient { split: 1, fee: false },
            v4v::FeeRecipient { split: 2, fee: true },
        ],
        expected_splits: Ok(vec![49, 49, 2]),
    },
    case_6: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 100, fee: true },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_7: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 2, fee: true },
            v4v::FeeRecipient { split: 1, fee: true },
        ],
        expected_splits: Ok(vec![2, 1]),
    },
    case_8: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 10, fee: false },
            v4v::FeeRecipient { split: 20, fee: false },
            v4v::FeeRecipient { split: 30, fee: false },
            v4v::FeeRecipient { split: 40, fee: false },
        ],
        expected_splits: Ok(vec![1, 2, 3, 4]),
    },
    case_9: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 10, fee: false },
            v4v::FeeRecipient { split: 20, fee: false },
            v4v::FeeRecipient { split: 30, fee: false },
            v4v::FeeRecipient { split: 40, fee: false },
            v4v::FeeRecipient { split: 1, fee: true },
        ],
        expected_splits: Ok(vec![99, 198, 297, 396, 10]),
    },
    case_10: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 10, fee: false },
            v4v::FeeRecipient { split: 20, fee: false },
            v4v::FeeRecipient { split: 30, fee: false },
            v4v::FeeRecipient { split: 40, fee: false },
            v4v::FeeRecipient { split: 1, fee: true },
            v4v::FeeRecipient { split: 1, fee: true },
        ],
        expected_splits: Ok(vec![49, 98, 147, 196, 5, 5]),
    },
    // Change order:
    case_11: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 1, fee: true },
            v4v::FeeRecipient { split: 10, fee: false },
            v4v::FeeRecipient { split: 20, fee: false },
            v4v::FeeRecipient { split: 30, fee: false },
            v4v::FeeRecipient { split: 1, fee: true },
            v4v::FeeRecipient { split: 40, fee: false },
        ],
        expected_splits: Ok(vec![5, 49, 98, 147, 5, 196]),
    },
    case_12: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 100, fee: true },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_13: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 1, fee: true },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_14: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 101, fee: true },
        ],
        expected_splits: Err(v4v::FeeRecipientsToSplitsError::TotalFeeExceeds100),
    },
    case_15: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 100, fee: true },
            v4v::FeeRecipient { split: 1, fee: false },
        ],
        expected_splits: Err(v4v::FeeRecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist),
    },
    case_16: TestCase {
        recipients: vec![
            v4v::FeeRecipient { split: 50, fee: false },
            v4v::FeeRecipient { split: 40, fee: false },
            v4v::FeeRecipient { split: 3, fee: false },
            v4v::FeeRecipient { split: 2, fee: false },
            v4v::FeeRecipient { split: 2, fee: false },
            v4v::FeeRecipient { split: 1, fee: false },
            v4v::FeeRecipient { split: 2, fee: true },
        ],
        expected_splits: Ok(vec![50, 40, 3, 2, 2, 1, 2]),
    },
}
