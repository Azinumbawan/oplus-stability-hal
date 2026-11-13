#!/bin/bash
# Build the Rust port of the OPlus OplusProject (stability) HAL for aarch64
# Android, then verify the artifact offline. Never touches the device and
# never repacks.
#
# Output:
#   rust/target/aarch64-linux-android/release/vendor.oplus.hardware.stability.oplus_project-V1-service
set -euo pipefail

W=$(cd "$(dirname "$0")/.." && pwd)
cd "$W/rust"

echo "=== host unit tests (node adapters) ==="
cargo test --offline
echo

echo "=== cross-build aarch64-linux-android (release) ==="
cargo build --release --offline --target aarch64-linux-android --bins

OUT=target/aarch64-linux-android/release
SRC="$OUT/oplus_project_service"
DST="$OUT/vendor.oplus.hardware.stability.oplus_project-V1-service"
[[ -f "$SRC" ]] || { echo "build produced no binary" >&2; exit 1; }
install -m 0755 "$SRC" "$DST"

NDK=/opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64
echo
echo "=== built ==="
ls -la "$DST"
"$NDK/bin/llvm-readelf" -d "$DST" | grep NEEDED
"$NDK/bin/llvm-readelf" -p .comment "$DST" | head -5
echo BUILD_OK
