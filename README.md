A set of helper functions for dealing with value-for-value (V4V) calculations and transactions.

Modules include

- [pc20] for Podcasting 2.0-specific functions, including those related to [sat calculations](pc20::calc), [V4V payments](pc20::payments), and [sat forwarding](pc20::forwarding).
- [alby] for interacting with [Alby](https://getalby.com)'s API.

Check out [docs.rs](https://docs.rs/v4v) for all available functions.

## Example

```rust
let splits = vec![1, 98, 1];
let total_sats = 10;
// The crate ensures that
// - even after rounding, the total number of sats is preserved
// - if possible, everyone gets at least 1 sat (and thus their own TLV record)
assert_eq!(v4v::pc20::calc::compute_sat_recipients(&splits, total_sats), vec![1, 8, 1]);
```

## Install

```text
cargo add v4v
```

## Contribute

Please feel free to contribute by submitting a PR on [GitHub](https://github.com/rssblue/v4v).
