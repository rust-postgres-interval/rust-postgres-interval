# Rust-Postgres-Interval
A interval type for the postgres driver.

# Contributing

There is a separate document on how to contribute to this repo [here](CONTRIBUTING.md)

## Overview
Rust-Postgres-Interval is dedicated datatype for the postgres interval type.

```rust
extern crate pg_interval;

use pg_interval::Interval;

fn main() {
    let interval = Interval::from_postgres(
        "1 years 1 months 1 days 1 hours"
    ).unwrap();
    let output = interval.to_iso_8601();
    assert_eq!(String::from("P1Y1M1DT1H"), output);

    let interval = Interval::from_std_duration(
        std::time::Duration::from_secs(3_661)
    ).unwrap();
    assert_eq!(Interval::new(0, 0, 3_661_000_000), interval);
}
```

## Features

Both features are on by default.

- `postgres`: `FromSql` and `ToSql` for the `postgres` driver.
- `chrono`: `Interval::from_duration` from chrono's `Duration`. Without it,
  the crate does not depend on chrono; `Interval::from_std_duration` is
  always available.
