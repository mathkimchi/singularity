cargo build --release
WINIT_UNIX_BACKEND=wayland RUST_BACKTRACE=full cargo run --bin singularity_sde
