#!/usr/bin/env bash
# Local equivalents of this repo's own heavy CI jobs, run by
# genai-rs/.github scripts/local-ci.sh after the shared checks.
set -euo pipefail

# ci.yml `no-default-features` job (Test (no default features)).
cargo +stable test --no-default-features
