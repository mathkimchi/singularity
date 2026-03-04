# cargo build --release
# cargo build # for some reason, without this, `./target/release/fortune_teller` doesn't update
# WINIT_UNIX_BACKEND=wayland RUST_BACKTRACE=full cargo run --bin singularity_sde

RUST_BACKTRACE=1 cargo run --bin main
