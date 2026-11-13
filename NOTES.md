# OplusStability — pure-Rust OPlus stability HAL

`vendor.oplus.hardware.stability.oplus_project.IOplusProject` (V1) for the
X6739 (MT6893). Reconstructed from the shipped OPlus artifacts, then rewritten
in Rust with no C or C++ code and no prebuilt library.

## Contract

- Descriptor: `vendor.oplus.hardware.stability.oplus_project.IOplusProject`
- Instance: `vendor.oplus.hardware.stability.oplus_project.IOplusProject/default`
- Interface version: 1
- Interface hash: `87177176cb812e29538fd78965a31f1f545d2baa`
- 33 methods, transaction codes 1..33 in declaration order
  (`aidl/vendor/oplus/hardware/stability/oplus_project/IOplusProject.aidl`),
  plus the reserved pair `getInterfaceVersion` (16777215 → int) and
  `getInterfaceHash` (16777214 → String).
- Each reply is a status header (OK) followed by the method return value;
  in-parameters are read in declaration order. The interface token is
  consumed by libbinder before `onTransact`.

## Provenance of the reconstruction

| Artifact | Path | SHA-256 |
|---|---|---|
| Stock service (stripped, aarch64) | `erofs/vendor/bin/hw/vendor.oplus.hardware.stability.oplus_project-V1-service` (pre-staging) | `7ac91baf9edad7b98b9c8df6898630a7d5f7230bee0ddd7fcdfc94195103c44d` |
| AIDL NDK backend (V1), shipped | `erofs/vendor/lib64/vendor.oplus.hardware.stability.oplus_project-V1-ndk_platform.so` | `316f34ef5fb346112ad4d6af1543b5aaf912db780a86a4834d559f06cf0d6f1a` |

The method order was verified against the shipped `BnOplusProject`
dispatch and the Java stub in `services.jar`.

## X6739 node adaptation

The OPlus kernel nodes the shipped HAL read do not exist on this board;
every method keeps the OPlus path as the primary candidate and this
kernel's MTK/Transsion equivalent as the fallback. The first candidate
that yields data (reads) or accepts the write (writes) wins.

The shipped behaviour the adapter preserves:

- `strtol` prefix parsing (`"5.10.198 (gcc)"` → 5) is implemented as
  `parse_i32_prefix`, because Rust's `str::parse` rejects numeric prefixes.
- `readAgingData` returns exactly five ints from the five battery nodes,
  absent nodes reading as 0.
- `saveAgingData` rejects a length above 4096 or one that disagrees with
  the array, then writes the values comma-joined to the theia aging node
  pair.
- `get_opluspoff_reason`/`get_opluspon_reason` return empty for a nonzero
  argument independently of the node state, matching the shipped branch.
- `get_force_reboot`/`get_shutdown_detect` answer the literal string
  `null` when no candidate yields data.
- `get_phoenix` reports `"1"` for the boot reasons `fastboot`, `download`,
  `recovery` and `bootloader`.
- `get_pmic_info` composes `vbat=<mV> ibat=<uA> temp=<10x degC>` from the
  battery supply nodes when the OPlus pmic_monitor node is absent.

## Build and stage

```bash
scripts/build_rust.sh    # host unit tests + aarch64 cross-build
scripts/stage_vendor.sh  # copies the binary into erofs/vendor/bin/hw/
```

The crate pins rustc via `rust/rust-toolchain.toml` (1.99.1). The link uses
the ROM tree's real `libbinder_ndk.so` because the service needs the
platform-only `AServiceManager_addService` / `ABinderProcess_*` entry points
(see `rust/.cargo/config.toml`).

## Layout

- `aidl/` — the interface contract (method order is load-bearing).
- `rust/src/core.rs` — every method against its node set, host-testable
  (root override + in-memory properties).
- `rust/src/wire.rs` — the hand-rolled `libbinder_ndk` C API binding and
  parcel helpers.
- `rust/src/hal.rs` — the `onTransact` dispatch for all 33 methods plus the
  reserved pair.
- `rust/src/bin/oplus_project_service.rs` — registration and the thread pool.

The C++ reconstruction the port replaced is preserved in the OOS root's
history.
