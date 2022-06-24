#!/bin/bash

set -ex

cargo build --release --target x86_64-unknown-linux-musl --bin sync_server
cargo build --release --target x86_64-unknown-linux-musl --bin track_warehouse

scp ../target/x86_64-unknown-linux-musl/release/sync_server user@192.168.1.26:/home/user/bak/tom/fy_admin/sync_server/
