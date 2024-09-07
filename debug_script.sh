cargo build --release
# cargo run --bin singularity_manager
cargo test --package singularity_common --lib -- tests::dylib_test --exact --show-output