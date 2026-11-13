// OplusProject HAL service entry point (Rust port).
//
// Startup recovered from the shipped binary and its init rc
// (erofs/vendor/etc/init/vendor.oplus.hardware.stability.oplus_project-V1-service.rc):
//
//   ABinderProcess_startThreadPool();
//   name   = "vendor.oplus.hardware.stability.oplus_project.IOplusProject/default";
//   status = AServiceManager_addService(binder, name);
//   ABinderProcess_joinThreadPool();
//
// The log tag is "OplusStabilityHAL". OPLUS_PROJECT_INSTANCE overrides the
// instance suffix so a rebuilt HAL can register beside the shipped one for
// A/B comparison; empty/unset keeps "/default".

#[path = "../wire.rs"]
mod wire;
#[path = "../hal.rs"]
mod hal;

use std::ffi::CString;
use std::os::raw::c_char;

use hal::{on_create, on_destroy, on_transact, DESCRIPTOR, LOG_TAG};
use wire::{
    log, AIBinder_Class_define, AIBinder_new, ABinderProcess_joinThreadPool,
    ABinderProcess_startThreadPool, AServiceManager_addService, LOG_LEVEL_ERROR, LOG_LEVEL_INFO,
};

fn main() {
    // SAFETY: configures this process's binder thread pool; no arguments.
    unsafe { ABinderProcess_startThreadPool() };

    let descriptor = CString::new(DESCRIPTOR).unwrap();
    // SAFETY: descriptor is NUL-terminated; the handlers match the typedefs.
    let class = unsafe {
        AIBinder_Class_define(
            descriptor.as_ptr() as *const c_char,
            on_create,
            on_destroy,
            on_transact,
        )
    };
    if class.is_null() {
        log(LOG_LEVEL_ERROR, LOG_TAG, "AIBinder_Class_define failed");
        std::process::exit(1);
    }

    // SAFETY: class was just defined; on_create builds the service state
    // itself, so no args are needed.
    let binder = unsafe { AIBinder_new(class, std::ptr::null_mut()) };
    if binder.is_null() {
        log(LOG_LEVEL_ERROR, LOG_TAG, "AIBinder_new failed");
        std::process::exit(1);
    }

    let suffix = std::env::var("OPLUS_PROJECT_INSTANCE").unwrap_or_default();
    let name = format!("{}/{}", DESCRIPTOR, if suffix.is_empty() { "default" } else { &suffix });
    let instance = CString::new(name.clone()).unwrap();
    // SAFETY: binder is a live AIBinder we own; instance is NUL-terminated.
    let status = unsafe { AServiceManager_addService(binder, instance.as_ptr() as *const c_char) };
    if status != 0 {
        log(
            LOG_LEVEL_ERROR,
            LOG_TAG,
            &format!("failed to register {} (status {})", name, status),
        );
        std::process::exit(1);
    }
    log(LOG_LEVEL_INFO, LOG_TAG, &format!("registered {}", name));

    // SAFETY: hands the main thread to the binder thread pool; returns only
    // when the process is shutting down.
    unsafe { ABinderProcess_joinThreadPool() };
}
