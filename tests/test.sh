cd safe-bugs/out-of-bound-index
cargo clean
#RUST_LOG=debug RUST_BACKTRACE=full ../../../target/debug/cargo-mir-checker mir-checker -- --domain interval --entry main --widening_delay 5 --narrowing_iteration 5 --deny_warnings
RUST_LOG=debug ../../../target/debug/cargo-mir-checker mir-checker -- --domain interval --entry main --widening_delay 5 --narrowing_iteration 5 --deny_warnings