#!/bin/sh
set -e

CROSS_RS_REV="7b79041c9278769eca57fae10c74741f5aa5c14b"
cargo install cross --git "https://github.com/cross-rs/cross.git" --rev "${CROSS_RS_REV}"
