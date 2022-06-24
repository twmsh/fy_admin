#!/bin/bash

set -e

cargo build --release --target x86_64-unknown-linux-musl --bin sync_server