#!/bin/bash
# Explicit binary copy only. A Project daemon/deployment is not assumed.
set -euo pipefail
: "${ROM_TREE:?set ROM_TREE to an unpacked tree}"
: "${PROJECT_RELATIVE_EXECUTABLE:?choose a reviewed vendor/bin/hw or odm/bin/hw V3 deployment path}"
case "$PROJECT_RELATIVE_EXECUTABLE" in
    vendor/bin/hw/vendor.oplus.hardware.stability.oplus_project-V3-service|odm/bin/hw/vendor.oplus.hardware.stability.oplus_project-V3-service) ;;
    *) echo "unrecognized Project V3 deployment path" >&2; exit 1 ;;
esac
W=$(cd "$(dirname "$0")/.." && pwd)
SRC="${PROJECT_ARTIFACT:-$W/rust/target/aarch64-linux-android/release/vendor.oplus.hardware.stability.oplus_project-V3-service}"
[[ -f "$SRC" ]] || { echo "missing artifact" >&2; exit 1; }

canonical_directory() {
    local resolved
    [[ -d "$1" ]] || return 1
    if command -v realpath >/dev/null 2>&1 && resolved=$(realpath -e -- "$1" 2>/dev/null); then
        [[ -d "$resolved" ]] || return 1
        printf '%s\n' "$resolved"
    elif command -v readlink >/dev/null 2>&1 && resolved=$(readlink -f -- "$1" 2>/dev/null); then
        [[ -d "$resolved" ]] || return 1
        printf '%s\n' "$resolved"
    else
        return 1
    fi
}

# An explicitly selected root may itself be a symlink; its physical target is
# the boundary. Internal links must resolve within that boundary.
ROM_ROOT=$(canonical_directory "$ROM_TREE") || { echo "missing or unresolvable ROM root" >&2; exit 1; }
DEST_PARENT=$(canonical_directory "${ROM_ROOT%/}/${PROJECT_RELATIVE_EXECUTABLE%/*}") || { echo "missing or unresolvable deployment directory" >&2; exit 1; }
case "$DEST_PARENT/" in
    "${ROM_ROOT%/}/"*) ;;
    *) echo "deployment directory escapes ROM root" >&2; exit 1 ;;
esac
DEST="$DEST_PARENT/${PROJECT_RELATIVE_EXECUTABLE##*/}"
[[ ! -L "$DEST" ]] || { echo "destination is a symlink" >&2; exit 1; }
[[ ! -e "$DEST" || -f "$DEST" ]] || { echo "destination is not a regular file" >&2; exit 1; }
# Ordinary symlink escapes are rejected; concurrent rename races are not an
# atomic fd-relative operation and still require a trusted, quiescent tree.
install -m 0755 -- "$SRC" "$DEST"
echo "Copied $DEST; no init, manifest, policy, fs_config or image changes were made."
