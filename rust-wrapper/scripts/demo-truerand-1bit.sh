#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

DEMO_DATASET="truerand_1bit.bin" \
DEMO_BITS="1" \
DEMO_LABEL="NIST true random 1-bit sample" \
"${SCRIPT_DIR}/demo.sh"
