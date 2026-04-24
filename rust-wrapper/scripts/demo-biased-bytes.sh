#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

DEMO_DATASET="biased-random-bytes.bin" \
DEMO_BITS="8" \
DEMO_LABEL="NIST biased random byte sample" \
"${SCRIPT_DIR}/demo.sh"
