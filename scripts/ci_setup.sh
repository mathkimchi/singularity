#!/usr/bin/env bash
set -euo pipefail
{
  sudo apt-get update
  sudo apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev libfontconfig1-dev libxkbcommon-dev \
    libudev-dev libseat-dev libgbm-dev libinput-dev libpixman-1-dev \
    libwayland-dev libegl1-mesa-dev libgl1-mesa-dev libvulkan-dev \
    libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxcb1-dev
  cargo fetch --locked
} >/tmp/ci_setup.log 2>&1 || { tail -n 30 /tmp/ci_setup.log; exit 1; }
echo "ci_setup ok"
