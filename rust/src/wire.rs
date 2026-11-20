// Real libbinder_ndk C APIs handle tokens, status headers and UTF-16 transport.
#![allow(dead_code)]

use oplus_project_service::protocol::{checked_length, consumed, MAX_INT_ARRAY, MAX_STRING_BYTES};
pub use oplus_project_service::protocol::{
    STATUS_BAD_VALUE, STATUS_UNEXPECTED_NULL, STATUS_UNKNOWN_TRANSACTION,
};
use std::ffi::{c_char, c_void, CString};

#[repr(C)]
pub struct AIBinder {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AIBinder_Class {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AParcel {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AStatus {
    _private: [u8; 0],
}
pub type BinderStatus = i32;
pub type TransactionCode = u32;
pub const STATUS_OK: i32 = 0;
pub const STATUS_NO_MEMORY: i32 = -12;
pub const TRANSACTION_GET_INTERFACE_VERSION: u32 = 0xffffff;
pub const TRANSACTION_GET_INTERFACE_HASH: u32 = 0xfffffe;
pub const LOG_LEVEL_INFO: i32 = 4;
pub const LOG_LEVEL_ERROR: i32 = 6;
pub type OnCreate = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
pub type OnDestroy = unsafe extern "C" fn(*mut c_void);
pub type OnTransact = unsafe extern "C" fn(*mut AIBinder, u32, *const AParcel, *mut AParcel) -> i32;

extern "C" {
    pub fn AIBinder_Class_define(
        descriptor: *const c_char,
        create: OnCreate,
        destroy: OnDestroy,
        transact: OnTransact,
    ) -> *mut AIBinder_Class;
    pub fn AIBinder_new(class: *const AIBinder_Class, args: *mut c_void) -> *mut AIBinder;
    pub fn AIBinder_getUserData(binder: *mut AIBinder) -> *mut c_void;
    pub fn AIBinder_getCallingUid() -> u32;
    pub fn AIBinder_markVintfStability(binder: *mut AIBinder);
    pub fn AServiceManager_addService(binder: *mut AIBinder, instance: *const c_char) -> i32;
    pub fn ABinderProcess_startThreadPool();
    pub fn ABinderProcess_joinThreadPool();
    fn AParcel_writeInt32(parcel: *mut AParcel, value: i32) -> i32;
    fn AParcel_readInt32(parcel: *const AParcel, value: *mut i32) -> i32;
    fn AParcel_writeBool(parcel: *mut AParcel, value: bool) -> i32;
    fn AParcel_writeString(parcel: *mut AParcel, value: *const c_char, length: i32) -> i32;
    fn AParcel_readString(
        parcel: *const AParcel,
        data: *mut c_void,
        allocator: Option<unsafe extern "C" fn(*mut c_void, i32, *mut *mut c_char) -> bool>,
    ) -> i32;
    fn AParcel_readInt32Array(
        parcel: *const AParcel,
        data: *mut c_void,
        allocator: Option<unsafe extern "C" fn(*mut c_void, i32, *mut *mut i32) -> bool>,
    ) -> i32;
    fn AParcel_writeInt32Array(parcel: *mut AParcel, values: *const i32, length: i32) -> i32;
    fn AParcel_getDataPosition(parcel: *const AParcel) -> i32;
    fn AParcel_getDataSize(parcel: *const AParcel) -> i32;
    fn AParcel_writeStatusHeader(parcel: *mut AParcel, status: *const AStatus) -> i32;
    fn AStatus_newOk() -> *mut AStatus;
    fn AStatus_fromExceptionCode(exception: i32) -> *mut AStatus;
    fn AStatus_delete(status: *mut AStatus);
    fn __android_log_print(priority: i32, tag: *const c_char, format: *const c_char, ...) -> i32;
}

fn check(status: i32) -> Result<(), i32> {
    if status == 0 {
        Ok(())
    } else {
        Err(status)
    }
}

pub fn log(priority: i32, tag: &str, message: &str) {
    if let (Ok(tag), Ok(message)) = (CString::new(tag), CString::new(message)) {
        unsafe {
            __android_log_print(priority, tag.as_ptr(), c"%s".as_ptr(), message.as_ptr());
        }
    }
}

pub fn enforce_no_data_avail(parcel: *const AParcel) -> Result<(), i32> {
    consumed(unsafe { AParcel_getDataPosition(parcel) }, unsafe {
        AParcel_getDataSize(parcel)
    })
}

fn write_status(out: *mut AParcel, exception: Option<i32>) -> Result<(), i32> {
    let status = unsafe {
        match exception {
            Some(e) => AStatus_fromExceptionCode(e),
            None => AStatus_newOk(),
        }
    };
    if status.is_null() {
        return Err(STATUS_NO_MEMORY);
    }
    let result = unsafe { AParcel_writeStatusHeader(out, status) };
    unsafe { AStatus_delete(status) };
    check(result)
}

pub fn write_ok_status(out: *mut AParcel) -> Result<(), i32> {
    write_status(out, None)
}
pub fn write_exception(out: *mut AParcel, exception: i32) -> Result<(), i32> {
    write_status(out, Some(exception))
}
pub fn write_i32(out: *mut AParcel, value: i32) -> Result<(), i32> {
    check(unsafe { AParcel_writeInt32(out, value) })
}
pub fn write_bool(out: *mut AParcel, value: bool) -> Result<(), i32> {
    check(unsafe { AParcel_writeBool(out, value) })
}

pub fn write_string(out: *mut AParcel, value: &str) -> Result<(), i32> {
    if value.len() > MAX_STRING_BYTES {
        return Err(STATUS_BAD_VALUE);
    }
    let value = CString::new(value).map_err(|_| STATUS_BAD_VALUE)?;
    check(unsafe { AParcel_writeString(out, value.as_ptr(), value.as_bytes().len() as i32) })
}

pub fn write_i32_array(out: *mut AParcel, values: &[i32]) -> Result<(), i32> {
    if values.len() > MAX_INT_ARRAY {
        return Err(STATUS_BAD_VALUE);
    }
    check(unsafe { AParcel_writeInt32Array(out, values.as_ptr(), values.len() as i32) })
}

pub fn read_i32(parcel: *const AParcel) -> Result<i32, i32> {
    let mut value = 0;
    check(unsafe { AParcel_readInt32(parcel, &mut value) })?;
    Ok(value)
}

#[derive(Default)]
struct StringCtx {
    bytes: Vec<u8>,
    null: bool,
    error: Option<i32>,
}

unsafe extern "C" fn string_allocator(
    data: *mut c_void,
    length: i32,
    out: *mut *mut c_char,
) -> bool {
    let ctx = &mut *data.cast::<StringCtx>();
    match checked_length(length, MAX_STRING_BYTES + 1, true) {
        Ok(None) => {
            ctx.null = true;
            // NDK passes no output-buffer pointer for a null value.
            true
        }
        Ok(Some(length)) => {
            if out.is_null() {
                ctx.error = Some(STATUS_BAD_VALUE);
                return false;
            }
            if ctx.bytes.try_reserve_exact(length).is_err() {
                ctx.error = Some(STATUS_NO_MEMORY);
                return false;
            }
            ctx.bytes.resize(length, 0);
            *out = ctx.bytes.as_mut_ptr().cast();
            true
        }
        Err(error) => {
            ctx.error = Some(error);
            false
        }
    }
}

pub fn read_string(parcel: *const AParcel) -> Result<String, i32> {
    let mut ctx = StringCtx::default();
    let status = unsafe {
        AParcel_readString(
            parcel,
            (&mut ctx as *mut StringCtx).cast(),
            Some(string_allocator),
        )
    };
    if let Some(error) = ctx.error {
        return Err(error);
    }
    check(status)?;
    if ctx.null {
        return Err(STATUS_UNEXPECTED_NULL);
    }
    if ctx.bytes.pop() != Some(0) || ctx.bytes.contains(&0) {
        return Err(STATUS_BAD_VALUE);
    }
    String::from_utf8(ctx.bytes).map_err(|_| STATUS_BAD_VALUE)
}

#[derive(Default)]
struct ArrayCtx {
    values: Vec<i32>,
    null: bool,
    error: Option<i32>,
}

unsafe extern "C" fn array_allocator(data: *mut c_void, length: i32, out: *mut *mut i32) -> bool {
    let ctx = &mut *data.cast::<ArrayCtx>();
    match checked_length(length, MAX_INT_ARRAY, false) {
        Ok(None) => {
            ctx.null = true;
            true
        }
        Ok(Some(length)) => {
            if length != 0 && out.is_null() {
                ctx.error = Some(STATUS_BAD_VALUE);
                return false;
            }
            if ctx.values.try_reserve_exact(length).is_err() {
                ctx.error = Some(STATUS_NO_MEMORY);
                return false;
            }
            ctx.values.resize(length, 0);
            if !out.is_null() {
                *out = ctx.values.as_mut_ptr();
            }
            true
        }
        Err(error) => {
            ctx.error = Some(error);
            false
        }
    }
}

pub fn read_i32_array(parcel: *const AParcel) -> Result<Vec<i32>, i32> {
    let mut ctx = ArrayCtx::default();
    let status = unsafe {
        AParcel_readInt32Array(
            parcel,
            (&mut ctx as *mut ArrayCtx).cast(),
            Some(array_allocator),
        )
    };
    if let Some(error) = ctx.error {
        return Err(error);
    }
    check(status)?;
    if ctx.null {
        return Err(STATUS_UNEXPECTED_NULL);
    }
    Ok(ctx.values)
}
