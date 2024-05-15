cd unit-tests/alloc-test
cargo clean
../../../target/debug/cargo-mir-checker mir-checker -v -- --domain interval --entry main --widening_delay 5 --narrowing_iteration 5 --deny_warnings