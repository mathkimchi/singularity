# cargo build --release
# WINIT_UNIX_BACKEND=wayland RUST_BACKTRACE=full cargo run --bin singularity_manager
# cargo test --package singularity_common --lib -- tests::dylib_test --exact --show-output
# cargo test --package singularity_ui --lib -- test --show-output
# cargo run --package singularity_ui --bin demo
# RUST_BACKTRACE=full cargo run --package singularity_ui --example ab_glyph_demo
# cd singularity_standard_tabs ; cargo expand demo
cd singularity_common ; cargo expand tab::packets # -Z macro-backtrace