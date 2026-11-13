// The cross-platform half of the OplusProject HAL port.
//
// Every IOplusProject method against its node set lives here so the adapter
// behaviour is unit-testable on the host (see core.rs). The binder-facing
// half (wire.rs / hal.rs) is an Android-only binary in
// src/bin/oplus_project_service.rs.

pub mod core;
pub mod io_shim;
