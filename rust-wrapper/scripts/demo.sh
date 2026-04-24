#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cd "${CRATE_DIR}"
DEMO_DATASET="${DEMO_DATASET:-data.pi.bin}" \
DEMO_BITS="${DEMO_BITS:-8}" \
DEMO_LABEL="${DEMO_LABEL:-Largest bundled NIST sample}" \
cargo run --example demo
