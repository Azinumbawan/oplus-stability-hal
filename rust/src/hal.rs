// ColorOS IOplusProject V3. Decode the entire request before any node access.

use crate::wire::*;
use oplus_project_service::core::{self, Error, IdentityField, Project};
pub use oplus_project_service::protocol::{DESCRIPTOR, INTERFACE_HASH, INTERFACE_VERSION};
use std::ffi::c_void;
pub const LOG_TAG: &str = "OplusProjectHAL";

pub unsafe extern "C" fn on_create(_args: *mut c_void) -> *mut c_void {
    Box::into_raw(Box::new(Project::new())).cast()
}

pub unsafe extern "C" fn on_destroy(data: *mut c_void) {
    if !data.is_null() {
        drop(Box::from_raw(data.cast::<Project>()));
    }
}

enum Args {
    None,
    Int(i32),
    Text(String),
    Aging(i32, Vec<i32>, i32),
    Theia(i32, String),
}

fn decode(code: u32, input: *const AParcel) -> Result<Args, i32> {
    let args = match code {
        9..=11 | 26 | 27 => Args::Int(read_i32(input)?),
        28 => Args::Aging(read_i32(input)?, read_i32_array(input)?, read_i32(input)?),
        29..=31 | 35 => Args::Text(read_string(input)?),
        32 => Args::Theia(read_i32(input)?, read_string(input)?),
        1..=35 | TRANSACTION_GET_INTERFACE_VERSION | TRANSACTION_GET_INTERFACE_HASH => Args::None,
        _ => return Err(STATUS_UNKNOWN_TRANSACTION),
    };
    enforce_no_data_avail(input)?;
    Ok(args)
}

pub unsafe extern "C" fn on_transact(
    binder: *mut AIBinder,
    code: u32,
    input: *const AParcel,
    output: *mut AParcel,
) -> i32 {
    dispatch(binder, code, input, output)
        .err()
        .unwrap_or(STATUS_OK)
}

unsafe fn dispatch(
    binder: *mut AIBinder,
    code: u32,
    input: *const AParcel,
    out: *mut AParcel,
) -> Result<(), i32> {
    let args = decode(code, input)?;
    if code == TRANSACTION_GET_INTERFACE_VERSION {
        return int_reply(out, Ok(INTERFACE_VERSION));
    }
    if code == TRANSACTION_GET_INTERFACE_HASH {
        return string_reply(out, Ok(INTERFACE_HASH.to_owned()));
    }
    let data = AIBinder_getUserData(binder);
    if data.is_null() {
        return Err(STATUS_UNEXPECTED_NULL);
    }
    let project = &*data.cast::<Project>();
    let uid = AIBinder_getCallingUid();
    match (code, args) {
        (1, Args::None) => int_reply(out, project.identity(IdentityField::Engineering)),
        (2, Args::None) => string_reply(out, core::read_native("/proc/force_reboot")),
        (3, Args::None) => int_reply(out, core::read_native_int("/sys/systeminfo/ftmmode")),
        (4, Args::None) => string_reply(out, core::read_native("/proc/sys/kernel/hung_task_kill")),
        (5, Args::None) => string_reply(
            out,
            core::read_native("/sys/devices/platform/pinctrl/mt_gpio"),
        ),
        (6, Args::None) => string_reply(out, core::read_native("/proc/oplusVersion/ocp")),
        (7, Args::None) => string_reply(out, core::read_native("/proc/opbootfrom")),
        (8, Args::None) => int_reply(out, project.identity(IdentityField::Operator)),
        (9, Args::Int(id)) => string_reply(out, core::read_pmic("/sys/pmic_info/ocp_status", id)),
        (10, Args::Int(id)) => string_reply(out, core::read_pmic("/sys/pmic_info/poff_reason", id)),
        (11, Args::Int(id)) => string_reply(out, core::read_pmic("/sys/pmic_info/pon_reason", id)),
        (12, Args::None) => int_reply(out, project.identity(IdentityField::Pcb)),
        (13, Args::None) => string_reply(out, core::read_native("/proc/phoenix")),
        (14, Args::None) => string_reply(out, core::read_native("/sys/pmic_info/pmic_monitor")),
        (15, Args::None) => string_reply(out, core::read_native("/sys/power/poff_reason")),
        (16, Args::None) => string_reply(out, core::read_native("/sys/power/pon_reason")),
        (17, Args::None) => int_reply(out, project.identity(IdentityField::Project)),
        (18, Args::None) => int_reply(out, project.identity(IdentityField::Rf)),
        (19, Args::None) => int_reply(out, core::read_native_int("/proc/sau_pwknum")),
        (20, Args::None) => string_reply(out, core::get_serial_id(uid)),
        (21, Args::None) => string_reply(out, core::read_native("/proc/shutdown_detect")),
        (22, Args::None) => bool_reply(out, core::is_ufs_devices()),
        (23, Args::None) => {
            string_reply(out, core::read_native("/vendor/lib/modules/modules.load"))
        }
        (24, Args::None) => string_reply(
            out,
            core::read_native("/vendor/lib/modules/modules.blocklist"),
        ),
        (25, Args::None) => string_reply(out, core::read_native("/proc/modules")),
        (26, Args::Int(_)) => error_reply(out, Error::Unsupported),
        (27, Args::Int(id)) => string_reply(out, core::read_theia_node(id)),
        (28, Args::Aging(selector, data, length)) => {
            if selector < 0 || length < 0 || length as usize != data.len() {
                return error_reply(out, Error::InvalidArgument);
            }
            bool_reply(out, core::unsupported_mutation(uid))
        }
        (29..=31 | 35, Args::Text(_text)) => bool_reply(out, core::unsupported_mutation(uid)),
        (32, Args::Theia(_id, _text)) => bool_reply(out, core::unsupported_mutation(uid)),
        (33, Args::None) => string_reply(
            out,
            core::read_native("/sys/class/devfreq/mtk-dvfsrc-devfreq/cur_freq"),
        ),
        (34, Args::None) => error_reply(out, Error::Unsupported),
        _ => Err(STATUS_BAD_VALUE),
    }
}

fn error_reply(out: *mut AParcel, error: Error) -> Result<(), i32> {
    write_exception(
        out,
        match error {
            Error::Unsupported => -7,      // EX_UNSUPPORTED_OPERATION
            Error::PermissionDenied => -1, // EX_SECURITY
            Error::InvalidArgument => -3,  // EX_ILLEGAL_ARGUMENT
        },
    )
}

fn int_reply(out: *mut AParcel, value: Result<i32, Error>) -> Result<(), i32> {
    match value {
        Ok(v) => {
            write_ok_status(out)?;
            write_i32(out, v)
        }
        Err(e) => error_reply(out, e),
    }
}
fn string_reply(out: *mut AParcel, value: Result<String, Error>) -> Result<(), i32> {
    match value {
        Ok(v) => {
            write_ok_status(out)?;
            write_string(out, &v)
        }
        Err(e) => error_reply(out, e),
    }
}
fn bool_reply(out: *mut AParcel, value: Result<bool, Error>) -> Result<(), i32> {
    match value {
        Ok(v) => {
            write_ok_status(out)?;
            write_bool(out, v)
        }
        Err(e) => error_reply(out, e),
    }
}
