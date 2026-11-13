# OplusStability

Pure-Rust implementation of the OPlus stability AIDL HAL
(`vendor.oplus.hardware.stability.oplus_project.IOplusProject` V1) for the
Infinix GT 10 Pro (X6739, MT6893).

Reconstructed from the shipped OPlus binaries and rewritten in Rust: no C or
C++ source, no prebuilt library. The binder transport is hand-rolled against
`libbinder_ndk`'s stable C API at the exact wire format the generated clients
use, and the interface hash is served verbatim.

See [NOTES.md](NOTES.md) for the contract, the node-adaptation table and the
build instructions.
