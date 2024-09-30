# cargo build --release
# WINIT_UNIX_BACKEND=x11 RUST_BACKTRACE=full cargo run --bin singularity_manager
# WINIT_UNIX_BACKEND=wayland RUST_BACKTRACE=full cargo run --bin singularity_manager
# cargo test --package singularity_common --lib -- tests::dylib_test --exact --show-output
# cargo run --package singularity_ui --bin demo
cargo run --package singularity_ui --example demo