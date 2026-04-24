#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

DEMO_DATASET="normal.bin" \
DEMO_BITS="8" \
DEMO_LABEL="NIST normal random byte sample" \
"${SCRIPT_DIR}/demo.sh"
