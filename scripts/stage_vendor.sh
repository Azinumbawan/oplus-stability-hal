#!/bin/bash
# Stage the built stability HAL service into an unpacked Android vendor tree.
#
# The service is installed under vendor/bin/hw/ at the path referenced by the
# HAL's init rc and VINTF fragment. That path is already covered by the tree's
# fs_config and file_contexts, so nothing needs regenerating.
#
# This only copies a file. It does not build or flash images, and does not
# touch fs_config or file_contexts.
set -euo pipefail

: "${ROM_TREE:?set ROM_TREE to the root of the unpacked vendor tree}"

W=$(cd "$(dirname "$0")/.." && pwd)
SRC="$W/rust/target/aarch64-linux-android/release/vendor.oplus.hardware.stability.oplus_project-V1-service"

GRN=$'\033[32m'; RED=$'\033[31m'; NC=$'\033[0m'

[[ -f "$SRC" ]] || { echo -e "${RED}missing $SRC: run scripts/build_rust.sh first${NC}"; exit 1; }

live="$ROM_TREE/vendor/bin/hw/vendor.oplus.hardware.stability.oplus_project-V1-service"
install -m 0755 "$SRC" "$live"
echo -e "${GRN}staged${NC} $live"
echo "done"
