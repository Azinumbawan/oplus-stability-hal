# Oplus Project HAL

Rust compatibility implementation of `vendor.oplus.hardware.stability.oplus_project.IOplusProject/default`, based on the reverse-engineered Oplus HAL in [Xia-s-Projekt/oplus-stability-hal](https://github.com/Xia-s-Projekt/oplus-stability-hal).

This fork targets the V3 interface: 35 methods, hash `3e738561c0bfdc622a1c84face254caf0c887c8f`. It provides project identity and selected diagnostics, not the full Oplus stability framework. It is an independent implementation, not official Oplus source.

## Behavior

- Identity comes from native `/proc/oplusVersion/` nodes or a trusted `/vendor/etc/oplus_project.conf` mapping. Device-model, kernel-version and SIM digits are not used as substitutes for Oplus identifiers.
- Configuration must match `ro.hardware`. It is size-limited, root-owned and not writable by other users. Unknown identity fields remain unsupported.
- Available diagnostic nodes are read directly. Missing data is not replaced with unrelated charging or recovery controls.
- Serial access requires a root or system Binder caller. Serial values are bounded, validated and never logged.
- Mutations, Theia controls, aging persistence and OP1 record storage are unsupported.

The Binder service uses explicit VINTF stability, bounded argument parsing and AIDL error replies. Matching interface metadata does not establish compatibility with every firmware release.

## Build

Requirements: installed Rust 1.95 or newer, the `aarch64-linux-android` target, Android NDK, and the target platform's actual `libbinder_ndk.so`. The default cross-build API is 35.

```sh
export ANDROID_NDK_ROOT=/path/to/android-ndk
export LIBBINDER_NDK=/path/to/platform/lib64/libbinder_ndk.so
bash scripts/build_rust.sh
```

`OPLUS_PLATFORM_LIB_DIR` may supply the platform library directory instead. The build does not install toolchains, run the HAL or modify a device.

## Tests

```sh
cargo +stable test --manifest-path rust/Cargo.toml --lib --locked
cargo +stable clippy --manifest-path rust/Cargo.toml --lib -- -D warnings
```

Tests use in-memory fixtures. They do not inspect real identifiers or exercise Android Binder interoperability.

## Integration

Confirm that the target framework needs this service before adding it. Supply the correct init service, V3 VINTF declaration, SELinux policy and verified identity mapping. Do not infer a service requirement from a shared library's presence alone.

The optional staging helper requires an explicit ROM tree and reviewed executable path. It copies only the binary; it does not configure policy, rebuild images or flash partitions. Use a trusted, quiescent extraction tree. Historical upstream prebuilts do not contain this fork's changes.

## Attribution

The upstream reverse-engineering and initial implementation are by Xia-s-Projekt and its contributors. This fork adds interface alignment, input validation and conservative platform integration. Existing copyright and license notices remain applicable.
