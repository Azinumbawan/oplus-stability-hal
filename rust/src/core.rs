// IOplusProject is an identity/diagnostic interface, not a kernel-control shim.

use crate::{io_shim, protocol::privileged};

pub const CONFIG_PATH: &str = "/vendor/etc/oplus_project.conf";
pub const MAX_CONFIG_BYTES: usize = 4096;
pub const MAX_NATIVE_BYTES: usize = 65536;
pub const MAX_SERIAL_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    PermissionDenied,
    InvalidArgument,
}

pub fn decimal_i32(text: &str) -> Option<i32> {
    let text = text.trim();
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

#[derive(Debug, Clone, Copy)]
pub enum IdentityField {
    Engineering,
    Operator,
    Pcb,
    Project,
    Rf,
}

impl IdentityField {
    fn index(self) -> usize {
        self as usize
    }
    fn node(self) -> &'static str {
        match self {
            Self::Engineering => "/proc/oplusVersion/engVersion",
            Self::Operator => "/proc/oplusVersion/operatorName",
            Self::Pcb => "/proc/oplusVersion/pcbVersion",
            Self::Project => "/proc/oplusVersion/prjName",
            Self::Rf => "/proc/oplusVersion/RFType",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct IdentityConfig {
    values: [Option<i32>; 5],
}

impl IdentityConfig {
    pub fn parse(text: &str, hardware: &str) -> Result<Self, Error> {
        if text.len() > MAX_CONFIG_BYTES || hardware.is_empty() || text.contains('\0') {
            return Err(Error::InvalidArgument);
        }
        let mut config = Self::default();
        let mut seen = std::collections::HashSet::new();
        let mut matches_hardware = false;
        for line in text
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
        {
            let (key, value) = line.split_once('=').ok_or(Error::InvalidArgument)?;
            let (key, value) = (key.trim(), value.trim());
            if !seen.insert(key) {
                return Err(Error::InvalidArgument);
            }
            if key == "hardware" {
                if value.len() > 64
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                {
                    return Err(Error::InvalidArgument);
                }
                matches_hardware = value == hardware;
                continue;
            }
            let field = match key {
                "eng_version" => IdentityField::Engineering,
                "operator_name" => IdentityField::Operator,
                "pcb_version" => IdentityField::Pcb,
                "project" => IdentityField::Project,
                "rf_type" => IdentityField::Rf,
                _ => return Err(Error::InvalidArgument),
            };
            config.values[field.index()] = Some(decimal_i32(value).ok_or(Error::InvalidArgument)?);
        }
        if !matches_hardware {
            return Err(Error::InvalidArgument);
        }
        Ok(config)
    }
}

#[derive(Default)]
pub struct Project {
    identity: IdentityConfig,
}

impl Project {
    pub fn new() -> Self {
        let hardware = io_shim::property_get("ro.hardware").unwrap_or_default();
        let identity = io_shim::read_trusted_file(CONFIG_PATH, MAX_CONFIG_BYTES)
            .and_then(|text| IdentityConfig::parse(&text, &hardware).ok())
            .unwrap_or_default();
        Self { identity }
    }

    pub fn identity(&self, field: IdentityField) -> Result<i32, Error> {
        io_shim::read_file(field.node(), 128)
            .and_then(|s| decimal_i32(&s))
            .or(self.identity.values[field.index()])
            .ok_or(Error::Unsupported)
    }
}

pub fn read_native(path: &str) -> Result<String, Error> {
    io_shim::read_file(path, MAX_NATIVE_BYTES)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .ok_or(Error::Unsupported)
}

pub fn read_native_int(path: &str) -> Result<i32, Error> {
    io_shim::read_file(path, 128)
        .and_then(|s| decimal_i32(&s))
        .ok_or(Error::Unsupported)
}

pub fn read_pmic(path: &str, pmic_id: i32) -> Result<String, Error> {
    if pmic_id != 0 {
        return Err(Error::Unsupported);
    }
    read_native(path)
}

pub fn get_serial_id(uid: u32) -> Result<String, Error> {
    if !privileged(uid) {
        return Err(Error::PermissionDenied);
    }
    let serial = io_shim::read_trusted_file("/data/oplusVer/Serial_ID", MAX_SERIAL_BYTES)
        .or_else(|| io_shim::property_get("ro.vendor.oplus.radio.serialno"))
        .ok_or(Error::Unsupported)?;
    let serial = serial.trim();
    if serial.is_empty()
        || serial.len() > MAX_SERIAL_BYTES
        || ["unknown", "null"]
            .iter()
            .any(|s| serial.eq_ignore_ascii_case(s))
        || !serial
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(Error::Unsupported);
    }
    Ok(serial.to_owned())
}

pub fn read_theia_node(index: i32) -> Result<String, Error> {
    let node = match index {
        1 => "/proc/pwkMonitorParam",
        2 => "/proc/theiaPwkReport",
        3 => "/proc/blackSwitch",
        4 => "/proc/brightSwitch",
        _ => return Err(Error::Unsupported),
    };
    read_native(node)
}

pub fn is_ufs_devices() -> Result<bool, Error> {
    let driver = read_native("/sys/class/scsi_host/host0/proc_name")?;
    // An unknown storage driver's text is not evidence of a negative UFS identity.
    match driver.as_str() {
        "ufshcd" | "ufshcd-mtk" => Ok(true),
        _ => Err(Error::Unsupported),
    }
}

// No write grammar or OP1/aging storage contract is verified for this port.
// Even a privileged caller cannot opt into guessed kernel or battery writes.
pub fn unsupported_mutation(uid: u32) -> Result<bool, Error> {
    if privileged(uid) {
        Err(Error::Unsupported)
    } else {
        Err(Error::PermissionDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_model_kernel_soc_or_sim_identity_inference() {
        io_shim::mock_property("ro.product.model", "TECNO CK8n");
        io_shim::mock_property("ro.hardware", "mt6893");
        io_shim::mock_property("gsm.sim.operator.numeric", "310260");
        io_shim::mock_file("/proc/version", "Linux version 5.10.272");
        io_shim::mock_file("/proc/device-tree/model", "CK8n");
        let project = Project::new();
        for field in [
            IdentityField::Engineering,
            IdentityField::Pcb,
            IdentityField::Project,
            IdentityField::Rf,
            IdentityField::Operator,
        ] {
            assert_eq!(project.identity(field), Err(Error::Unsupported));
        }
    }

    #[test]
    fn trusted_typed_mapping_and_native_precedence() {
        io_shim::mock_property("ro.hardware", "mt6893");
        io_shim::mock_file(CONFIG_PATH, "hardware=mt6893\nproject=12345\neng_version=0\nrf_type=4\noperator_name=7\npcb_version=2");
        let project = Project::new();
        assert_eq!(project.identity(IdentityField::Project), Ok(12345));
        assert_eq!(project.identity(IdentityField::Engineering), Ok(0));
        io_shim::mock_file("/proc/oplusVersion/prjName", "24680\n");
        assert_eq!(project.identity(IdentityField::Project), Ok(24680));
    }

    #[test]
    fn mapping_rejects_wrong_hardware_duplicates_and_bad_numbers() {
        for text in [
            "hardware=mt6991\nproject=1",
            "hardware=mt6893\nproject=-1",
            "hardware=mt6893\nproject=2147483648",
            "hardware=mt6893\nproject=CK8n",
            "hardware=mt6893\nproject=1\nproject=2",
            "hardware=mt6893\nunknown=1",
        ] {
            assert!(IdentityConfig::parse(text, "mt6893").is_err(), "{text}");
        }
        assert_eq!(decimal_i32("1 extra"), None);
        assert_eq!(decimal_i32("+1"), None);
    }

    #[test]
    fn theia_never_reads_charger_fallbacks_and_mutations_are_disabled() {
        io_shim::mock_file("/sys/class/power_supply/tran-auto-test/cc1_status", "1");
        io_shim::mock_file("/sys/class/power_supply/tran-auto-test/aging", "9");
        assert_eq!(read_theia_node(3), Err(Error::Unsupported));
        assert_eq!(unsupported_mutation(0), Err(Error::Unsupported));
        assert_eq!(unsupported_mutation(1000), Err(Error::Unsupported));
        assert_eq!(unsupported_mutation(10001), Err(Error::PermissionDenied));
        assert!(io_shim::mock_reads()
            .iter()
            .all(|s| !s.contains("power_supply")));
    }

    #[test]
    fn unknown_storage_data_does_not_invent_false() {
        assert_eq!(is_ufs_devices(), Err(Error::Unsupported));
        io_shim::mock_file("/sys/class/scsi_host/host0/proc_name", "unverified-driver");
        assert_eq!(is_ufs_devices(), Err(Error::Unsupported));
        io_shim::mock_file("/sys/class/scsi_host/host0/proc_name", "ufshcd\n");
        assert_eq!(is_ufs_devices(), Ok(true));
    }

    #[test]
    fn serial_checks_uid_before_io_and_rejects_bad_content() {
        assert_eq!(get_serial_id(10001), Err(Error::PermissionDenied));
        assert!(io_shim::mock_reads().is_empty());
        io_shim::mock_file("/data/oplusVer/Serial_ID", "MOCK-ONLY_123\n");
        assert_eq!(get_serial_id(1000), Ok("MOCK-ONLY_123".to_owned()));
        io_shim::mock_file("/data/oplusVer/Serial_ID", "MOCK\nINJECTED");
        assert_eq!(get_serial_id(1000), Err(Error::Unsupported));
        io_shim::mock_file("/data/oplusVer/Serial_ID", "UNKNOWN");
        assert_eq!(get_serial_id(1000), Err(Error::Unsupported));
        io_shim::mock_file("/data/oplusVer/Serial_ID", &"x".repeat(65));
        assert_eq!(get_serial_id(1000), Err(Error::Unsupported));
    }
}
