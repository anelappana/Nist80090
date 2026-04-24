#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

DEMO_DATASET="rand8_short.bin" \
DEMO_BITS="8" \
DEMO_LABEL="NIST short 8-bit random sample" \
"${SCRIPT_DIR}/demo.sh"
