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
