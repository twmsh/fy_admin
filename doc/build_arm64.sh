#!/bin/bash

set -e

cargo build --release --target aarch64-unknown-linux-musl --bin sync_client
cargo build --release --target aarch64-unknown-linux-musl --bin box_agent
