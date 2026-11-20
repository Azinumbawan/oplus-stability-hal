// OplusProject HAL service entry point.
//
// Startup sequence:
//
//   ABinderProcess_startThreadPool();
//   name   = "vendor.oplus.hardware.stability.oplus_project.IOplusProject/default";
//   status = AServiceManager_addService(binder, name);
//   ABinderProcess_joinThreadPool();
//
// OPLUS_PROJECT_INSTANCE overrides the
// instance suffix; empty or unset keeps "/default".

#[path = "../hal.rs"]
mod hal;
#[path = "../wire.rs"]
mod wire;

use std::ffi::CString;
use std::os::raw::c_char;

use hal::{on_create, on_destroy, on_transact, DESCRIPTOR, LOG_TAG};
use wire::{
    log, ABinderProcess_joinThreadPool, ABinderProcess_startThreadPool, AIBinder_Class_define,
    AIBinder_markVintfStability, AIBinder_new, AServiceManager_addService, LOG_LEVEL_ERROR,
    LOG_LEVEL_INFO,
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

    // The manual transport does not process @VintfStability automatically.
    unsafe { AIBinder_markVintfStability(binder) };

    let suffix = std::env::var("OPLUS_PROJECT_INSTANCE").unwrap_or_default();
    if suffix.len() > 64
        || !suffix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        log(LOG_LEVEL_ERROR, LOG_TAG, "invalid instance suffix");
        std::process::exit(1);
    }
    let name = format!(
        "{}/{}",
        DESCRIPTOR,
        if suffix.is_empty() {
            "default"
        } else {
            &suffix
        }
    );
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
