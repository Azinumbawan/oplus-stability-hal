# OplusStability

A pure-Rust implementation of the OPlus stability AIDL HAL service.

## What it does

`OplusStability` registers
`vendor.oplus.hardware.stability.oplus_project.IOplusProject` (version 1) with
the Android service manager and serves its 33 methods plus the reserved
`getInterfaceVersion` and `getInterfaceHash` transactions. The methods report
stability, boot, power, and aging information from kernel nodes and system
properties, and expose setters for a small set of boot and power flags.

Each method has an ordered list of candidate nodes and properties; the first
candidate that yields data or accepts a write wins. The binder transport binds
the stable `libbinder_ndk` C API directly, so there is no generated C++ code and
no prebuilt library. The platform-specific core is host-testable through
fixture files and an in-memory property store.

## Layout

- `aidl/` — the AIDL interface definition. Method order defines transaction
  codes and is load-bearing.
- `rust/` — the Cargo crate. `src/core.rs` holds the method implementations,
  `src/wire.rs` and `src/hal.rs` the binder transport and transaction dispatch,
  and `src/bin/` the service entry point.
- `scripts/` — `build_rust.sh` builds and verifies the service;
  `stage_vendor.sh` installs it into an unpacked vendor tree.

## Requirements

- The Rust toolchain pinned in `rust/rust-toolchain.toml`, including the
  `aarch64-linux-android` target.
- An Android NDK providing `aarch64-linux-android35-clang` and `llvm-readelf`.
- The platform `libbinder_ndk.so` from the target Android tree, which provides
  the `AServiceManager_*` and `ABinderProcess_*` entry points used by the
  service.

## Prebuilt

[Get Prebuilt](https://github.com/Xia-s-Projekt/oplus-stability-hal/releases/latest) — AArch64 ODM archive containing the service under `odm/bin/hw/`, its init rc under `odm/etc/init/`, and the VINTF manifest under `odm/etc/vintf/manifest/`.

## Build

Host unit tests and the cross-compiled service are produced by one script:

```bash
ANDROID_NDK_ROOT=/path/to/ndk \
LIBBINDER_NDK=/path/to/platform/libbinder_ndk.so \
  scripts/build_rust.sh
```

The release binary is written to
`rust/target/aarch64-linux-android/release/vendor.oplus.hardware.stability.oplus_project-V1-service`.

## Staging

`stage_vendor.sh` copies the built service into `vendor/bin/hw/` of an unpacked
Android tree, at the path referenced by the HAL's init rc and VINTF fragment:

```bash
ROM_TREE=/path/to/unpacked/tree scripts/stage_vendor.sh
```

Staging only copies the binary. It does not build or flash images, and does not
regenerate `fs_config` or `file_contexts`.
