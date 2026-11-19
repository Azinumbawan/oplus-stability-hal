// The OplusProject HAL service: the onTransact dispatch for all 33
// IOplusProject methods plus the reserved getInterfaceVersion /
// getInterfaceHash pair.
//
// Method order follows the AIDL declaration order (transaction codes 1..33).
// Every reply is a status header (OK) followed by the method's return value;
// in-parameters are read from the incoming parcel in declaration order. The
// interface token is consumed by libbinder before onTransact.

use std::ffi::c_void;

use crate::wire::*;

use oplus_project_service::core;

pub const LOG_TAG: &str = "OplusStabilityHAL";

pub const DESCRIPTOR: &str = "vendor.oplus.hardware.stability.oplus_project.IOplusProject";
pub const INTERFACE_HASH: &str = "87177176cb812e29538fd78965a31f1f545d2baa";
pub const INTERFACE_VERSION: i32 = 1;

/// The service state. IOplusProject keeps no cross-call state: every method
/// is a node read or write.
pub struct Service;

pub unsafe extern "C" fn on_create(_args: *mut c_void) -> *mut c_void {
    let svc = Box::new(Service);
    Box::into_raw(svc) as *mut c_void
}

pub unsafe extern "C" fn on_destroy(user_data: *mut c_void) {
    if !user_data.is_null() {
        drop(Box::from_raw(user_data as *mut Service));
    }
}

pub unsafe extern "C" fn on_transact(
    _binder: *mut AIBinder,
    code: TransactionCode,
    in_parcel: *const AParcel,
    out_parcel: *mut AParcel,
) -> BinderStatus {
    let res = match code {
        1 => i32_method(out_parcel, core::get_eng_version()),
        2 => string_method(out_parcel, core::get_force_reboot()),
        3 => i32_method(out_parcel, core::get_ftmmode()),
        4 => string_method(out_parcel, core::get_hungtask()),
        5 => string_method(out_parcel, core::get_mt_gpio()),
        6 => string_method(out_parcel, core::get_ocp()),
        7 => string_method(out_parcel, core::get_opboot()),
        8 => i32_method(out_parcel, core::get_operator_name()),
        9 => match read_i32(in_parcel) {
            Ok(arg0) => string_method(out_parcel, core::get_oplusocp_status(arg0)),
            Err(rc) => Err(rc),
        },
        10 => match read_i32(in_parcel) {
            Ok(arg0) => string_method(out_parcel, core::get_opluspoff_reason(arg0)),
            Err(rc) => Err(rc),
        },
        11 => match read_i32(in_parcel) {
            Ok(arg0) => string_method(out_parcel, core::get_opluspon_reason(arg0)),
            Err(rc) => Err(rc),
        },
        12 => i32_method(out_parcel, core::get_pcb_version()),
        13 => string_method(out_parcel, core::get_phoenix()),
        14 => string_method(out_parcel, core::get_pmic_info()),
        15 => string_method(out_parcel, core::get_poff_reason()),
        16 => string_method(out_parcel, core::get_pon_reason()),
        17 => i32_method(out_parcel, core::get_project()),
        18 => i32_method(out_parcel, core::get_rf_type()),
        19 => i32_method(out_parcel, core::get_sau()),
        20 => string_method(out_parcel, core::get_serial_id()),
        21 => string_method(out_parcel, core::get_shutdown_detect()),
        22 => bool_method(out_parcel, core::is_ufs_devices()),
        23 => string_method(out_parcel, core::misc_module2_load()),
        24 => string_method(out_parcel, core::misc_module_blocklist()),
        25 => string_method(out_parcel, core::misc_read_linked_modules()),
        26 => match read_i32(in_parcel) {
            Ok(arg0) => i32_array_method(out_parcel, core::read_aging_data(arg0)),
            Err(rc) => Err(rc),
        },
        27 => match read_i32(in_parcel) {
            Ok(arg0) => string_method(out_parcel, core::read_theia_node(arg0)),
            Err(rc) => Err(rc),
        },
        28 => save_aging_data(in_parcel, out_parcel),
        29 => match read_string(in_parcel) {
            Ok(arg0) => bool_method(out_parcel, core::set_opboot(&arg0)),
            Err(rc) => Err(rc),
        },
        30 => match read_string(in_parcel) {
            Ok(arg0) => bool_method(out_parcel, core::set_phoenix(&arg0)),
            Err(rc) => Err(rc),
        },
        31 => match read_string(in_parcel) {
            Ok(arg0) => bool_method(out_parcel, core::set_shutdown_detect(&arg0)),
            Err(rc) => Err(rc),
        },
        32 => match (read_i32(in_parcel), read_string(in_parcel)) {
            (Ok(arg0), Ok(arg1)) => bool_method(out_parcel, core::write_theia_node(arg0, &arg1)),
            (_, Err(rc)) | (Err(rc), _) => Err(rc),
        },
        33 => string_method(out_parcel, core::get_dram_freq()),
        TRANSACTION_GET_INTERFACE_VERSION => i32_method(out_parcel, INTERFACE_VERSION),
        TRANSACTION_GET_INTERFACE_HASH => string_method(out_parcel, INTERFACE_HASH.to_string()),
        _ => {
            // Codes outside the interface return UNKNOWN_TRANSACTION; clients
            // use it to walk the interface chain.
            return -31;
        }
    };

    match res {
        Ok(()) => STATUS_OK,
        Err(rc) => rc,
    }
}

fn i32_method(out: *mut AParcel, value: i32) -> Result<(), BinderStatus> {
    write_ok_status(out)?;
    write_i32(out, value)
}

fn string_method(out: *mut AParcel, value: String) -> Result<(), BinderStatus> {
    write_ok_status(out)?;
    write_string(out, &value)
}

fn bool_method(out: *mut AParcel, value: bool) -> Result<(), BinderStatus> {
    write_ok_status(out)?;
    write_bool(out, value)
}

fn i32_array_method(out: *mut AParcel, values: Vec<i32>) -> Result<(), BinderStatus> {
    write_ok_status(out)?;
    write_i32_array(out, &values)
}

fn save_aging_data(in_parcel: *const AParcel, out: *mut AParcel) -> Result<(), BinderStatus> {
    let arg0 = read_i32(in_parcel)?;
    let arg1 = read_i32_array(in_parcel)?;
    let arg2 = read_i32(in_parcel)?;
    bool_method(out, core::save_aging_data(arg0, &arg1, arg2))
}
