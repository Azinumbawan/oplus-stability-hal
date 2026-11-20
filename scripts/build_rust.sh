#!/bin/bash
# Host mock tests, then an Android cross-build. Never executes the service.
set -euo pipefail
: "${ANDROID_NDK_ROOT:?set ANDROID_NDK_ROOT to an existing NDK}"
: "${LIBBINDER_NDK:?set LIBBINDER_NDK to the real target-platform libbinder_ndk.so}"
W=$(cd "$(dirname "$0")/.." && pwd)
cd "$W/rust"
NDK="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/${NDK_HOST_TAG:-linux-x86_64}"
export PATH="$NDK/bin:$PATH"
TOOLCHAIN=${RUST_TOOLCHAIN:-stable}
rustup run "$TOOLCHAIN" cargo test --offline --lib
rustup run "$TOOLCHAIN" cargo build --release --offline --target aarch64-linux-android --bin oplus_project_service
OUT="${CARGO_TARGET_DIR:-target}/aarch64-linux-android/release"
SRC="$OUT/oplus_project_service"
DST="$OUT/vendor.oplus.hardware.stability.oplus_project-V3-service"
[[ -f "$SRC" ]] || { echo "build produced no service" >&2; exit 1; }
install -m 0755 "$SRC" "$DST"
"$NDK/bin/llvm-readelf" -d "$DST"
echo "Built IOplusProject V3 only; init/VINTF/SELinux integration is not supplied."
