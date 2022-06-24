#!/bin/bash

set -ex

cargo build --release --target x86_64-unknown-linux-musl --bin sync_server