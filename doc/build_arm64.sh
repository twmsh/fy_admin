#!/bin/bash

set -ex

cargo build --release --target aarch64-unknown-linux-musl --bin sync_client
cargo build --release --target aarch64-unknown-linux-musl --bin box_agent

scp ../target/aarch64-unknown-linux-musl/release/sync_client linaro@192.168.1.220:/data/fy_admin/sync_client/

scp ../target/aarch64-unknown-linux-musl/release/box_agent linaro@192.168.1.220:/data/fy_admin/box_agent/

