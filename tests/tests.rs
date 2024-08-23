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
                    assert_eq!(v4v::pc20::calc::compute_sat_recipients(&$value.splits, $value.total_sats), $value.expected_sats);
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
    case_7: TestCase {
        splits: vec![99, 1, 1],
        total_sats: 10,
        expected_sats: vec![8, 1, 1],
    },
    case_8: TestCase {
        splits: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        total_sats: 10,
        expected_sats: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    },
    case_9: TestCase {
        splits: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        total_sats: 9,
        expected_sats: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    },
    case_10: TestCase {
        splits: vec![2, 2, 2, 2, 1, 2, 2, 2, 2, 2],
        total_sats: 9,
        expected_sats: vec![1, 1, 1, 1, 0, 1, 1, 1, 1, 1],
    },
    case_11: TestCase {
        splits: vec![1000000, 1000000, 1000000, 1000000, 1, 1000000, 1000000, 1000000, 1000000, 1000000],
        total_sats: 9,
        expected_sats: vec![1, 1, 1, 1, 0, 1, 1, 1, 1, 1],
    },
    case_12: TestCase {
        splits: vec![1000000, 1000000, 1000000, 1000000, 0, 1000000, 1000000, 1000000, 1000000, 1000000],
        total_sats: 9,
        expected_sats: vec![1, 1, 1, 1, 0, 1, 1, 1, 1, 1],
    },
    case_13: TestCase {
        splits: vec![1000000, 1000000, 1000000, 1000000, 0, 1000000, 1000000, 1000000, 1000000, 1000000],
        total_sats: 11,
        expected_sats: vec![2, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    },
    case_14: TestCase {
        splits: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        total_sats: 55,
        expected_sats: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    },
    case_15: TestCase {
        splits: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        total_sats: 56,
        expected_sats: vec![1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    },
    case_16: TestCase {
        splits: vec![0, 0, 0, 0, 0],
        total_sats: 6,
        expected_sats: vec![2, 1, 1, 1, 1],
    },
    case_17: TestCase {
        splits: vec![u64::MAX],
        total_sats: 100,
        expected_sats: vec![100],
    },
    case_18: TestCase {
        splits: vec![u64::MAX, u64::MAX],
        total_sats: 100,
        expected_sats: vec![50, 50],
    },
    case_19: TestCase {
        splits: vec![u64::MAX, 1, u64::MAX],
        total_sats: 100,
        expected_sats: vec![50, 1, 49],
    },
    case_20: TestCase {
        splits: vec![u64::MAX, 0, u64::MAX],
        total_sats: 100,
        expected_sats: vec![50, 1, 49],
    },
    case_21: TestCase {
        splits: vec![],
        total_sats: 100,
        expected_sats: vec![],
    },
    case_22: TestCase {
        splits: vec![],
        total_sats: 0,
        expected_sats: vec![],
    },
    case_23: TestCase {
        splits: vec![1],
        total_sats: 0,
        expected_sats: vec![0],
    },
    case_24: TestCase {
        splits: vec![u64::MAX, u64::MAX],
        total_sats: 0,
        expected_sats: vec![0, 0],
    },
}

macro_rules! compute_sat_recipients_generic_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<compute_sat_recipients_generic_ $name>]() {
                    #[derive(Debug, PartialEq, Clone)]
                    struct MyStruct {
                        split: u64,
                    }

                    impl v4v::pc20::calc::HasSplit for MyStruct {
                        fn get_split(&self) -> u64 {
                            self.split
                        }

                        fn set_split(&mut self, split: u64) {
                            self.split = split;
                        }
                    }

                    struct TestCase {
                        recipients: Vec<MyStruct>,
                        total_sats: u64,
                        expected_sats: Vec<u64>,
                    }
                    assert_eq!(v4v::pc20::calc::compute_sat_recipients_generic(&$value.recipients, $value.total_sats), $value.expected_sats);
                }
            )*
        }
    }
}

compute_sat_recipients_generic_tests! {
    case_1: TestCase {
        recipients: vec![
            MyStruct { split: 50 },
            MyStruct { split: 50 },
        ],
        total_sats: 1000,
        expected_sats: vec![500, 500],
    },
}

macro_rules! fee_recipients_to_splits_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<fee_recipients_to_splits_ $name>]() {
                    struct TestCase {
                        recipients: Vec<v4v::pc20::calc::GenericRecipient>,
                        expected_splits: Result<Vec<u64>, v4v::pc20::calc::RecipientsToSplitsError>,
                    }
                    assert_eq!(v4v::pc20::calc::fee_recipients_to_splits(&$value.recipients), $value.expected_splits);
                }
            )*
        }
    }
}

fee_recipients_to_splits_tests! {
    case_1: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
        ],
        expected_splits: Ok(vec![1, 1]),
    },
    case_2: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 2 },
        ],
        expected_splits: Ok(vec![49, 49, 2]),
    },
    case_3: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![99, 99, 2]),
    },
    case_4: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![49, 49, 1, 1]),
    },
    case_5: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 2 },
        ],
        expected_splits: Ok(vec![49, 49, 2]),
    },
    case_6: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 100 },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_7: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 2 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![2, 1]),
    },
    case_8: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 40 },
        ],
        expected_splits: Ok(vec![1, 2, 3, 4]),
    },
    case_9: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 40 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![99, 198, 297, 396, 10]),
    },
    case_10: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 40 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![49, 98, 147, 196, 5, 5]),
    },
    // Change order:
    case_11: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 10 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 20 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 30 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 40 },
        ],
        expected_splits: Ok(vec![5, 49, 98, 147, 5, 196]),
    },
    case_12: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 100 },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_13: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![1]),
    },
    case_14: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 101 },
        ],
        expected_splits: Err(v4v::pc20::calc::RecipientsToSplitsError::TotalFeeExceeds100),
    },
    case_15: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 100 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
        ],
        expected_splits: Err(v4v::pc20::calc::RecipientsToSplitsError::FeeIs100ButNonFeeRecipientsExist),
    },
    case_16: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 50 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 40 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 3 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 2 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 2 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 2 },
        ],
        expected_splits: Ok(vec![50, 40, 3, 2, 2, 1, 2]),
    },
    case_17: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 99999 },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
        ],
        expected_splits: Ok(vec![99999, 1]),
    },
    case_18: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 0 },
            v4v::pc20::calc::GenericRecipient::PercentageBased{ percentage: 0 },
        ],
        expected_splits: Ok(vec![0, 0]),
    },
    case_19: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: u64::MAX },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: u64::MAX },
        ],
        expected_splits: Ok(vec![1, 1]),
    },
    case_20: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: u64::MAX },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
        ],
        expected_splits: Ok(vec![u64::MAX, 1]),
    },
    case_21: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: u64::MAX },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        expected_splits: Ok(vec![99, 1]),
    },
    case_22: TestCase {
        recipients: vec![
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: u64::MAX },
            v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: 1 },
            v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: 1 },
        ],
        // Third recipient is *approximately* 1% of the total.
        expected_splits: Ok(vec![u64::MAX, 1, u64::MAX/99+16]),
    },
}

macro_rules! fee_recipients_to_splits_generic_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<fee_recipients_to_splits_generic_ $name>]() {
                    #[derive(Debug, PartialEq, Clone)]
                    struct MyStruct {
                        split: u64,
                        fee: bool,
                    }

                    impl v4v::pc20::calc::HasSplit for MyStruct {
                        fn get_split(&self) -> u64 {
                            self.split
                        }

                        fn set_split(&mut self, split: u64) {
                            self.split = split;
                            self.fee = false;
                        }
                    }

                    impl From<MyStruct> for v4v::pc20::calc::GenericRecipient {
                        fn from(value: MyStruct) -> Self {
                            if value.fee {
                                v4v::pc20::calc::GenericRecipient::PercentageBased { percentage: value.split }
                            } else {
                                v4v::pc20::calc::GenericRecipient::ShareBased { num_shares: value.split }
                            }
                        }
                    }

                    struct TestCase {
                        recipients: Vec<MyStruct>,
                        expected_recipients: Result<Vec<MyStruct>, v4v::pc20::calc::RecipientsToSplitsError>,
                    }
                    assert_eq!(v4v::pc20::calc::fee_recipients_to_splits_generic(&$value.recipients), $value.expected_recipients);
                }
            )*
        }
    }
}

fee_recipients_to_splits_generic_tests! {
    case_1: TestCase {
        recipients: vec![
            MyStruct { split: 50, fee: false },
            MyStruct { split: 50, fee: false },
        ],
        expected_recipients: Ok(vec![
            MyStruct { split: 1, fee: false },
            MyStruct { split: 1, fee: false },
        ]),
    },
    case_2: TestCase {
        recipients: vec![
            MyStruct { split: 50, fee: false },
            MyStruct { split: 50, fee: false },
            MyStruct { split: 1, fee: true },
        ],
        expected_recipients: Ok(vec![
            MyStruct { split: 99, fee: false },
            MyStruct { split: 99, fee: false },
            MyStruct { split: 2, fee: false },
        ]),
    },
}

macro_rules! use_remote_splits_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<use_remote_splits_ $name>]() {
                    struct TestCase {
                        local_splits: Vec<u64>,
                        remote_splits: Vec<u64>,
                        remote_percentage: u64,
                        expected_local_splits: Vec<u64>,
                        expected_remote_splits: Vec<u64>,
                    }
                    assert_eq!(v4v::pc20::calc::use_remote_splits(&$value.local_splits, &$value.remote_splits, $value.remote_percentage), ($value.expected_local_splits, $value.expected_remote_splits));
                }
            )*
        }
    }
}

use_remote_splits_tests! {
    case_1: TestCase {
        local_splits: vec![100],
        remote_splits: vec![],
        remote_percentage: 0,
        expected_local_splits: vec![1],
        expected_remote_splits: vec![],
    },
    case_2: TestCase {
        local_splits: vec![],
        remote_splits: vec![100],
        remote_percentage: 0,
        expected_local_splits: vec![],
        expected_remote_splits: vec![1],
    },
    case_3: TestCase {
        local_splits: vec![],
        remote_splits: vec![],
        remote_percentage: 0,
        expected_local_splits: vec![],
        expected_remote_splits: vec![],
    },
    case_4: TestCase {
        local_splits: vec![1, 2, 3],
        remote_splits: vec![4, 5, 6],
        remote_percentage: 0,
        expected_local_splits: vec![1, 2, 3],
        expected_remote_splits: vec![0, 0, 0],
    },
    case_5: TestCase {
        local_splits: vec![1, 2, 3],
        remote_splits: vec![4, 5, 6],
        remote_percentage: 100,
        expected_local_splits: vec![0, 0, 0],
        expected_remote_splits: vec![4, 5, 6],
    },
    case_6: TestCase {
        local_splits: vec![1, 2, 3],
        remote_splits: vec![4, 5, 6],
        remote_percentage: 50,
        expected_local_splits: vec![5, 10, 15],
        expected_remote_splits: vec![8, 10, 12],
    },
    case_7: TestCase {
        local_splits: vec![1, 2, 3],
        remote_splits: vec![4, 5, 6],
        remote_percentage: 1000,
        expected_local_splits: vec![0, 0, 0],
        expected_remote_splits: vec![4, 5, 6],
    },
}

macro_rules! use_remote_splits_generic_tests {
    ($($name:ident: $value:expr,)*) => {
        paste::item! {
            $(
                #[test]
                fn [<use_remote_splits_generic_ $name>]() {
                    #[derive(Debug, PartialEq, Clone)]
                    struct MyStruct {
                        split: u64,
                    }
                    impl v4v::pc20::calc::HasSplit for MyStruct {
                        fn get_split(&self) -> u64 {
                            self.split
                        }

                        fn set_split(&mut self, split: u64) {
                            self.split = split;
                        }
                    }

                    struct TestCase {
                        local_values: Vec<MyStruct>,
                        remote_values: Vec<MyStruct>,
                        remote_percentage: u64,
                        expected_values: Vec<MyStruct>,
                    }
                    assert_eq!(v4v::pc20::calc::use_remote_splits_generic(&$value.local_values, &$value.remote_values, $value.remote_percentage), $value.expected_values);
                }
            )*
        }
    }
}

use_remote_splits_generic_tests! {
    case_1: TestCase {
        local_values: vec![MyStruct { split: 100 }],
        remote_values: vec![MyStruct { split: 50 }],
        remote_percentage: 40,
        expected_values: vec![MyStruct { split: 3 }, MyStruct { split: 2 }],
    },
}
