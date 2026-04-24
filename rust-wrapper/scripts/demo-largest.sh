#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

DEMO_DATASET="data.pi.bin" \
DEMO_BITS="8" \
DEMO_LABEL="Largest bundled NIST sample" \
"${SCRIPT_DIR}/demo.sh"
