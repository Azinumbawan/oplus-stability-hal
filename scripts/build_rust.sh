#!/bin/bash
# Run the host unit tests, then build and verify the aarch64 Android
# stability HAL service.
set -euo pipefail

: "${ANDROID_NDK_ROOT:?set ANDROID_NDK_ROOT to the Android NDK root}"
: "${LIBBINDER_NDK:?set LIBBINDER_NDK to the platform libbinder_ndk.so to link against}"

W=$(cd "$(dirname "$0")/.." && pwd)
cd "$W/rust"

NDK="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64"
export PATH="$NDK/bin:$PATH"

echo "=== host unit tests (node adapters) ==="
cargo test --offline
echo

echo "=== cross-build aarch64-linux-android (release) ==="
export RUSTFLAGS="-C link-arg=-Wl,--no-undefined -C link-arg=-Wl,-z,relro,-z,now -C link-arg=-Wl,--build-id=none -C link-arg=-llog -C link-arg=$LIBBINDER_NDK"
cargo build --release --offline --target aarch64-linux-android --bins

OUT=target/aarch64-linux-android/release
SRC="$OUT/oplus_project_service"
DST="$OUT/vendor.oplus.hardware.stability.oplus_project-V1-service"
[[ -f "$SRC" ]] || { echo "build produced no binary" >&2; exit 1; }
install -m 0755 "$SRC" "$DST"

echo
echo "=== built ==="
ls -la "$DST"
llvm-readelf -d "$DST" | grep NEEDED
llvm-readelf -p .comment "$DST" | head -5
echo BUILD_OK
