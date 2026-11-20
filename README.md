# OplusProject

A deliberately partial, read-only Rust identity/diagnostic provider for
`vendor.oplus.hardware.stability.oplus_project.IOplusProject/default`.
It is not a full Oplus stability daemon or an emulation of missing Oplus drivers.

## Contract

- ColorOS donor interface version **3**, hash
  `3e738561c0bfdc622a1c84face254caf0c887c8f`.
- All 35 transaction IDs and Java argument/return shapes were checked against
  the donor's compiled Stub constants and actual Proxy transact instructions.
- This fork's existing IDs 1-33 match donor V3. The actual donor V1 has only
  32 user methods; V3 adds 33 `String get_dram_freq()`, 34
  `String read_op1_record_info()`, and 35
  `boolean write_op1_record_info(String op1_logs_data)`.
- Metadata and successful replies use normal AIDL status/return ordering.
  Missing or unverified functionality returns `EX_UNSUPPORTED_OPERATION`, not
  fabricated zero/false, an empty value, or the literal string `"null"`.
- This is a manual NDK transport, not a generated or frozen Soong API tree.
  Original nullable annotations cannot be inferred from Java descriptors alone.
  The supplied non-null input contract rejects null strings/arrays, invalid
  lengths, embedded NUL/invalid UTF-8, and unconsumed trailing request data.
- The binder is explicitly marked VINTF-stable. Unknown transactions return
  NDK `STATUS_UNKNOWN_TRANSACTION` (`-74`). Registration still requires a real
  platform service-manager/VINTF/SELinux deployment.

## Identity Sources

Engineering, operator enumeration, PCB, project and RF identity use only strict
nonnegative decimal values from these exact native nodes:

```text
/proc/oplusVersion/engVersion
/proc/oplusVersion/operatorName
/proc/oplusVersion/pcbVersion
/proc/oplusVersion/prjName
/proc/oplusVersion/RFType
```

If a native value is unavailable, an integrator may supply a verified mapping
in `/vendor/etc/oplus_project.conf`. Android accepts only a regular, root-owned,
non-symlink file that is not group/world writable. It is read once at startup,
bounded to 4096 bytes, and its `hardware` must exactly match `ro.hardware`.
Duplicate/unknown keys, negative/non-integer/overflowing values and a hardware
mismatch reject the entire mapping. Native values take precedence.

The following is a valid empty mapping, not CK8n identity data:

```ini
hardware=mt6893
# Add ONLY independently verified Oplus decimal enum/identity values:
# project=...
# pcb_version=...
# eng_version=...
# rf_type=...
# operator_name=...
```

Omit unknown fields. A legitimate, explicitly supplied zero is accepted; zero
is never invented as an unknown identity sentinel. There is no model/SoC/kernel
digit extraction, SIM MCC/MNC-to-operator-enum conversion, JSON identity
database, immutable sibling interface, or project setter. Live native reads
and the immutable startup mapping are not an atomic hardware identity snapshot.

## Supported And Unsupported Operations

Available native diagnostic nodes are returned as bounded UTF-8 text, without
battery substitutions, invented JSON, boot-reason-to-Phoenix translations,
cmdline/device-tree binary parsing, or guessed unit conversions. The PMIC
adapter supports only index 0; other PMIC indices are unsupported. Module-named
getters read lists only and do not load modules or run commands.
Storage detection accepts a known UFS host-driver token; an absent or unverified
driver does not produce a fabricated negative UFS identity.

Theia reads use only `/proc/pwkMonitorParam`, `/proc/theiaPwkReport`,
`/proc/blackSwitch` and `/proc/brightSwitch` for IDs 1-4. No charger CC, aging or
`ita_setting` fallback is read or written. Aging-array and OP1 storage semantics
are unverified, so those operations are unsupported. **All mutations remain
disabled**, including native boot/Theia writes: even a privileged caller cannot
enable a guessed grammar. Mutating calls from other UIDs receive `EX_SECURITY`.

`get_serial_ID` permits only Binder UIDs 0 and 1000, before any source access.
Its file source is root-owned/non-symlink/non-writable by other users and read
with a 64-byte limit; the Oplus serial property is an alternate source. Only
ASCII alphanumeric, dot, underscore and hyphen content is returned. Values are
never logged or included in configuration. This does not replace SELinux policy.

CK8n's inspected Oplus identity and boot/Theia nodes are absent. Consequently,
identity needs a trusted verified mapping and those diagnostics remain
unsupported. Charger-node existence is not evidence of equivalent functionality.
A donor V3 library/permission alias does not prove a Project daemon is required;
the observed binder-log oneshot is not such a daemon.

## Host Tests

From `rust/`, using an already installed stable Rust 1.95 or newer:

```powershell
rustup run stable cargo test --offline --lib --target-dir "C:\Users\Azareel\AppData\Local\Temp\kilo\oplus-project-host-tests"
```

Tests use thread-local in-memory node/property fixtures. They do not open host
`/proc`, inspect identifiers, create Unix symlinks, execute Binder/HAL code, or
write device nodes. Toolchain configuration does not auto-request Android targets.

## Android Build And Deployment

Use an existing Android target and NDK compiler. `build.rs` requires
`LIBBINDER_NDK=/absolute/path/libbinder_ndk.so`, or
`OPLUS_PLATFORM_LIB_DIR=/absolute/platform/lib64`, and links that exact platform
library rather than the public NDK stub. Platform manager/process APIs and
`AParcel_getDataSize` require an appropriate real runtime (minimum API 31).
Cross builds target API 35 by default; actual symbol/namespace availability on
the ROM still needs verification. Windows callers can override
`CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` with the existing NDK clang `.cmd`.

The Bash helper uses `ANDROID_NDK_ROOT`, `LIBBINDER_NDK`, optional
`NDK_HOST_TAG`/`RUST_TOOLCHAIN`/`CARGO_TARGET_DIR`, and only tests/cross-builds:

```bash
scripts/build_rust.sh
```

It produces `vendor.oplus.hardware.stability.oplus_project-V3-service`.
No init rc, manifest, policy, identity configuration or image is generated.
`stage_vendor.sh` copies a binary only when `ROM_TREE` and an explicit reviewed
`PROJECT_RELATIVE_EXECUTABLE` select the V3 executable under `vendor/bin/hw/` or
`odm/bin/hw/`. `PROJECT_ARTIFACT` can select an external build artifact.
It does not establish a daemon, modify fs_config/file_contexts, or flash/repack.
Upstream V1 prebuilts are not artifacts of this V3 implementation.
