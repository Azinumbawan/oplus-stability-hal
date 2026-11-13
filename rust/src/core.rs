// The OplusProject HAL core: every IOplusProject method against its node set.
//
// Primary candidate is the OPlus-native path the shipped HAL read; the
// fallback is this board's MTK/Transsion equivalent. The first candidate
// that yields data (reads) or accepts the write (writes) wins. Behaviour is
// translated from the C++ reconstruction, which recorded the evidence for
// every fallback in `re/kernel_nodes_equiv.md` (kept in the OOS root's
// history).

use crate::io_shim;

pub fn read_first(cands: &[&str]) -> String {
    for c in cands {
        if let Some(s) = io_shim::read_file(c) {
            let t = s.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    String::new()
}

pub fn write_first(cands: &[&str], value: &str) -> bool {
    for c in cands {
        if io_shim::write_file(c, value) {
            return true;
        }
    }
    false
}

pub fn prop_get_any(names: &[&str]) -> String {
    for n in names {
        if let Some(v) = io_shim::property_get(n) {
            if !v.is_empty() {
                return v;
            }
        }
    }
    String::new()
}

/// Parses a leading numeric prefix the way the shipped C++ strtol does:
/// "5.10.198 (gcc)" -> 5, "42 units" -> 42, "" -> None. Rust's str::parse
/// rejects these because it wants the whole string.
fn parse_i32_prefix(rest: &str) -> Option<i32> {
    let bytes = rest.as_bytes();
    let mut end = 0;
    if !bytes.is_empty() && (bytes[0] == b'-' || bytes[0] == b'+') {
        end = 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == 0 || (end == 1 && (bytes[0] == b'-' || bytes[0] == b'+')) {
        return None;
    }
    rest[..end].parse::<i32>().ok()
}

pub fn first_int(s: &str) -> i32 {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b.is_ascii_digit() {
            return parse_i32_prefix(&s[i..]).unwrap_or(0);
        }
    }
    0
}

pub fn read_int_first(cands: &[&str]) -> i32 {
    for c in cands {
        if let Some(s) = io_shim::read_file(c) {
            let t = s.trim();
            if t.is_empty() {
                continue;
            }
            return parse_i32_prefix(t).unwrap_or(0);
        }
    }
    0
}

pub fn read_int_scan_first(cands: &[&str]) -> i32 {
    for c in cands {
        if let Some(s) = io_shim::read_file(c) {
            if let Some(v) = first_int_of(&s) {
                return v;
            }
        }
    }
    0
}

fn first_int_of(s: &str) -> Option<i32> {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b.is_ascii_digit() {
            return parse_i32_prefix(&s[i..]);
        }
    }
    None
}

pub fn read_contains(cands: &[&str], needle: &str) -> bool {
    for c in cands {
        if let Some(s) = io_shim::read_file(c) {
            if s.contains(needle) {
                return true;
            }
        }
    }
    false
}

pub fn read_after(cands: &[&str], marker: &str) -> String {
    for c in cands {
        if let Some(s) = io_shim::read_file(c) {
            if let Some(p) = s.find(marker) {
                let rest = &s[p + marker.len()..];
                let end = rest
                    .find(|ch: char| ch == ' ' || ch == '\t' || ch == '\n')
                    .unwrap_or(rest.len());
                return rest[..end].to_string();
            }
        }
    }
    String::new()
}

pub const THEIA_NODES: [[&str; 2]; 4] = [
    [
        "/proc/pwkMonitorParam",
        "/sys/class/power_supply/tran-auto-test/ita_setting",
    ],
    [
        "/proc/theiaPwkReport",
        "/sys/class/power_supply/tran-auto-test/aging",
    ],
    [
        "/proc/blackSwitch",
        "/sys/class/power_supply/tran-auto-test/cc1_status",
    ],
    [
        "/proc/brightSwitch",
        "/sys/class/power_supply/tran-auto-test/cc2_status",
    ],
];

fn theia_C(idx: usize) -> &'static [&'static str] {
    &THEIA_NODES[idx]
}

pub fn get_eng_version() -> i32 {
    read_int_scan_first(&["/proc/oplusVersion/engVersion", "/proc/version"])
}

pub fn get_force_reboot() -> String {
    let s = read_first(&["/proc/force_reboot", "/proc/device-tree/chosen/atag,two_sec_reboot"]);
    if s.is_empty() {
        "null".to_string()
    } else {
        s
    }
}

pub fn get_ftmmode() -> i32 {
    let s = read_first(&["/sys/systeminfo/ftmmode"]);
    if !s.is_empty() {
        return first_int(&s);
    }
    let mode = io_shim::property_get("ro.bootmode").unwrap_or_default();
    if mode.contains("ftm") {
        1
    } else {
        0
    }
}

pub fn get_hungtask() -> String {
    let s = read_first(&[
        "/proc/sys/kernel/hung_task_kill",
        "/proc/sys/kernel/hung_task_timeout_secs",
    ]);
    if !s.is_empty() {
        return s;
    }
    let t = read_first(&["/proc/sys/kernel/hung_task_panic"]);
    if t.is_empty() {
        String::new()
    } else {
        format!("timeout={} panic={}", s, t)
    }
}

pub fn get_mt_gpio() -> String {
    read_first(&["/sys/devices/platform/pinctrl/mt_gpio"])
}

pub fn get_ocp() -> String {
    read_first(&["/proc/oplusVersion/ocp"])
}

pub fn get_opboot() -> String {
    read_after(&["/proc/opbootfrom", "/proc/cmdline"], "bootopt=")
}

pub fn get_operator_name() -> i32 {
    let s = read_first(&["/proc/oplusVersion/operatorName"]);
    if !s.is_empty() {
        return first_int(&s);
    }
    first_int(&prop_get_any(&[
        "gsm.sim.operator.numeric",
        "gsm.operator.numeric",
    ]))
}

pub fn get_oplusocp_status(_arg0: i32) -> String {
    read_first(&["/sys/pmic_info/ocp_status"])
}

pub fn get_opluspoff_reason(arg0: i32) -> String {
    let s = read_first(&["/sys/pmic_info/poff_reason"]);
    if !s.is_empty() {
        return s;
    }
    if arg0 != 0 {
        return String::new();
    }
    read_first(&["/proc/device-tree/chosen/atag,shutdown_time"])
}

pub fn get_opluspon_reason(arg0: i32) -> String {
    let s = read_first(&["/sys/pmic_info/pon_reason"]);
    if !s.is_empty() {
        return s;
    }
    if arg0 != 0 {
        return String::new();
    }
    prop_get_any(&["vendor.oplus.boot.bootreason", "ro.boot.bootreason"])
}

pub fn get_pcb_version() -> i32 {
    read_int_scan_first(&["/proc/oplusVersion/pcbVersion", "/proc/device-tree/model"])
}

pub fn get_phoenix() -> String {
    let s = read_first(&["/proc/phoenix"]);
    if !s.is_empty() {
        return s;
    }
    let reason = prop_get_any(&["vendor.oplus.boot.bootreason", "ro.boot.bootreason"]);
    match reason.as_str() {
        "fastboot" | "download" | "recovery" | "bootloader" => "1".to_string(),
        _ => String::new(),
    }
}

pub fn get_pmic_info() -> String {
    let s = read_first(&["/sys/pmic_info/pmic_monitor"]);
    if !s.is_empty() {
        return s;
    }
    let vb = read_first(&["/sys/class/power_supply/battery/voltage_now"]);
    if vb.is_empty() {
        return String::new();
    }
    let ib = read_first(&["/sys/class/power_supply/battery/current_now"]);
    let temp = read_first(&["/sys/class/power_supply/battery/temp"]);
    format!("vbat={} ibat={} temp={}", vb, ib, temp)
}

pub fn get_poff_reason() -> String {
    let s = read_first(&["/sys/power/poff_reason"]);
    if !s.is_empty() {
        return s;
    }
    prop_get_any(&["vendor.oplus.boot.bootreason", "ro.boot.bootreason"])
}

pub fn get_pon_reason() -> String {
    let s = read_first(&["/sys/power/pon_reason"]);
    if !s.is_empty() {
        return s;
    }
    let reason = prop_get_any(&["vendor.oplus.boot.bootreason", "ro.boot.bootreason"]);
    if !reason.is_empty() {
        return reason;
    }
    read_after(&["/proc/cmdline"], "bootopt=")
}

pub fn get_project() -> i32 {
    let s = read_first(&["/proc/oplusVersion/prjName"]);
    if !s.is_empty() {
        return first_int(&s);
    }
    let p = first_int(&io_shim::property_get("ro.product.model").unwrap_or_default());
    if p != 0 {
        return p;
    }
    first_int(&read_first(&["/proc/device-tree/model"]))
}

pub fn get_rf_type() -> i32 {
    let s = read_first(&["/proc/oplusVersion/RFType", "/proc/device-tree/compatible"]);
    if !s.is_empty() {
        return first_int(&s);
    }
    first_int(&io_shim::property_get("ro.hardware").unwrap_or_default())
}

pub fn get_sau() -> i32 {
    read_int_scan_first(&["/proc/sau_pwknum"])
}

pub fn get_serial_id() -> String {
    let s = read_first(&["/data/oplusVer/Serial_ID"]);
    if !s.is_empty() {
        return s;
    }
    prop_get_any(&["ro.vendor.oplus.radio.serialno", "ro.serialno"])
}

pub fn get_shutdown_detect() -> String {
    let s = read_first(&[
        "/proc/shutdown_detect",
        "/proc/device-tree/chosen/atag,shutdown_time",
    ]);
    if s.is_empty() {
        "null".to_string()
    } else {
        s
    }
}

pub fn is_ufs_devices() -> bool {
    read_contains(
        &["/proc/devinfo/ufs", "/sys/class/scsi_host/host0/proc_name"],
        "ufs",
    )
}

pub fn misc_module2_load() -> String {
    read_first(&["/vendor/lib/modules/modules.load"])
}

pub fn misc_module_blocklist() -> String {
    read_first(&["/vendor/lib/modules/modules.blocklist"])
}

pub fn misc_read_linked_modules() -> String {
    read_first(&["/proc/modules"])
}

pub fn read_aging_data(_arg0: i32) -> Vec<i32> {
    vec![
        read_int_first(&["/sys/class/power_supply/tran-auto-test/bat_cycle"]),
        read_int_first(&["/sys/class/power_supply/tran-auto-test/battery_id"]),
        read_int_first(&["/sys/class/power_supply/tran-auto-test/aging"]),
        read_int_first(&["/sys/class/power_supply/battery/capacity"]),
        read_int_first(&["/sys/class/power_supply/battery/cycle_count"]),
    ]
}

pub fn read_theia_node(arg0: i32) -> String {
    match arg0 {
        1 => read_first(theia_C(0)),
        2 => read_first(theia_C(1)),
        3 => read_first(theia_C(2)),
        4 => read_first(theia_C(3)),
        _ => String::new(),
    }
}

pub fn save_aging_data(_arg0: i32, arg1: &[i32], arg2: i32) -> bool {
    if arg2 > 4096 || arg2 != arg1.len() as i32 {
        return false;
    }
    let mut v = String::new();
    for (i, x) in arg1.iter().enumerate() {
        if i > 0 {
            v.push(',');
        }
        v.push_str(&x.to_string());
    }
    write_first(
        &[
            "/proc/theiaPwkReport",
            "/sys/class/power_supply/tran-auto-test/aging",
        ],
        &v,
    )
}

pub fn set_opboot(_arg0: &str) -> bool {
    write_first(&["/proc/opbootfrom"], "1")
}

pub fn set_phoenix(_arg0: &str) -> bool {
    write_first(&["/proc/phoenix"], "1")
}

pub fn set_shutdown_detect(_arg0: &str) -> bool {
    write_first(&["/proc/shutdown_detect"], "1")
}

pub fn write_theia_node(arg0: i32, arg1: &str) -> bool {
    match arg0 {
        1 => write_first(theia_C(0), arg1),
        2 => write_first(theia_C(1), arg1),
        3 => write_first(theia_C(2), arg1),
        4 => write_first(theia_C(3), arg1),
        _ => false,
    }
}

pub fn get_dram_freq() -> String {
    read_first(&["/sys/class/devfreq/mtk-dvfsrc-devfreq/cur_freq"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup(name: &str) -> (PathBuf, std::sync::MutexGuard<'static, ()>) {
        let guard = io_shim::test_lock();
        let root = format!("/tmp/opencode/stab_fixture_{}", name);
        let _ = fs::remove_dir_all(&root);
        for d in [
            "proc/oplusVersion",
            "proc/devinfo",
            "proc/sys/kernel",
            "sys/class/power_supply/tran-auto-test",
            "sys/class/power_supply/battery",
            "sys/class/devfreq/mtk-dvfsrc-devfreq",
            "sys/systeminfo",
            "sys/pmic_info",
            "proc/device-tree/chosen",
        ] {
            fs::create_dir_all(format!("{}/{}", root, d)).unwrap();
        }
        io_shim::set_test_root(&root);
        (PathBuf::from(root), guard)
    }

    fn teardown(root: &PathBuf) {
        io_shim::clear_test_root();
        let _ = fs::remove_dir_all(root);
    }

    fn w(root: &PathBuf, rel: &str, v: &str) {
        fs::write(format!("{}/{}", root.display(), rel), v).unwrap();
    }

    #[test]
    fn eng_version_scans_int_from_version_line() {
        let (r, _guard) = setup("eng_version");
        w(&r, "proc/version", "Linux version 5.10.198 (gcc) #1 SMP\n");
        assert_eq!(get_eng_version(), 5);
        w(&r, "proc/oplusVersion/engVersion", "11\n");
        assert_eq!(get_eng_version(), 11);
        teardown(&r);
    }

    #[test]
    fn force_reboot_falls_back_to_null() {
        let (r, _guard) = setup("force_reboot");
        assert_eq!(get_force_reboot(), "null");
        w(&r, "proc/force_reboot", "1\n");
        assert_eq!(get_force_reboot(), "1");
        teardown(&r);
    }

    #[test]
    fn ftmmode_from_node_then_bootmode() {
        let (r, _guard) = setup("ftmmode");
        assert_eq!(get_ftmmode(), 0);
        w(&r, "sys/systeminfo/ftmmode", "ftm.mode=1\n");
        assert_eq!(get_ftmmode(), 1);
        teardown(&r);
    }

    #[test]
    fn hungtask_composes_timeout_and_panic() {
        let (r, _guard) = setup("hungtask");
        w(&r, "proc/sys/kernel/hung_task_panic", "1\n");
        assert_eq!(get_hungtask(), "timeout= panic=1");
        w(&r, "proc/sys/kernel/hung_task_timeout_secs", "120\n");
        assert_eq!(get_hungtask(), "120");
        teardown(&r);
    }

    #[test]
    fn opboot_reads_after_marker() {
        let (r, _guard) = setup("opboot");
        w(
            &r,
            "proc/cmdline",
            "console=ttyMSM0 bootopt=64S3N32P3 androidboot...\n",
        );
        assert_eq!(get_opboot(), "64S3N32P3");
        teardown(&r);
    }

    #[test]
    fn poff_and_pon_reason_branch_on_arg() {
        let (r, _guard) = setup("poff_pon");
        // No node present: arg0 == 0 falls back to the device-tree path,
        // arg0 != 0 returns empty.
        assert_eq!(get_opluspoff_reason(1), "");
        assert_eq!(get_opluspoff_reason(0), "");
        assert_eq!(get_opluspon_reason(1), "");
        w(&r, "sys/pmic_info/poff_reason", "uvlo\n");
        w(&r, "sys/pmic_info/pon_reason", "longkey\n");
        assert_eq!(get_opluspoff_reason(7), "uvlo");
        assert_eq!(get_opluspon_reason(7), "longkey");
        teardown(&r);
    }

    #[test]
    fn phoenix_maps_bootreasons() {
        let (r, _guard) = setup("phoenix");
        io_shim::property_set("vendor.oplus.boot.bootreason", "fastboot");
        assert_eq!(get_phoenix(), "1");
        io_shim::property_set("vendor.oplus.boot.bootreason", "normal");
        assert_eq!(get_phoenix(), "");
        w(&r, "proc/phoenix", "on\n");
        assert_eq!(get_phoenix(), "on");
        teardown(&r);
    }

    #[test]
    fn pmic_info_composes_battery_nodes() {
        let (r, _guard) = setup("pmic_info");
        w(&r, "sys/class/power_supply/battery/voltage_now", "4200000\n");
        w(&r, "sys/class/power_supply/battery/current_now", "300000\n");
        w(&r, "sys/class/power_supply/battery/temp", "310\n");
        assert_eq!(get_pmic_info(), "vbat=4200000 ibat=300000 temp=310");
        teardown(&r);
    }

    #[test]
    fn ufs_detection_by_content() {
        let (r, _guard) = setup("ufs");
        w(&r, "proc/devinfo/ufs", "ufs version 3.1\n");
        assert!(is_ufs_devices());
        w(&r, "proc/devinfo/ufs", "none\n");
        assert!(!is_ufs_devices());
        teardown(&r);
    }

    #[test]
    fn aging_data_reads_five_nodes() {
        let (r, _guard) = setup("aging");
        w(&r, "sys/class/power_supply/tran-auto-test/bat_cycle", "42\n");
        w(&r, "sys/class/power_supply/tran-auto-test/battery_id", "7\n");
        w(&r, "sys/class/power_supply/tran-auto-test/aging", "88\n");
        w(&r, "sys/class/power_supply/battery/capacity", "99\n");
        w(&r, "sys/class/power_supply/battery/cycle_count", "12\n");
        assert_eq!(read_aging_data(0), vec![42, 7, 88, 99, 12]);
        teardown(&r);
    }

    #[test]
    fn theia_switch_and_write_back() {
        let (r, _guard) = setup("theia");
        w(&r, "sys/class/power_supply/tran-auto-test/ita_setting", "ita=1\n");
        assert_eq!(read_theia_node(1), "ita=1");
        assert_eq!(read_theia_node(9), "");
        assert!(write_theia_node(1, "ita=2"));
        assert_eq!(read_theia_node(1), "ita=2");
        assert!(!write_theia_node(9, "x"));
        teardown(&r);
    }

    #[test]
    fn save_aging_data_validates_count_then_writes() {
        let (r, _guard) = setup("save_aging");
        assert!(!save_aging_data(0, &[1, 2], 3));
        assert!(!save_aging_data(0, &[1, 2], 4097));
        assert!(save_aging_data(0, &[4, 5], 2));
        assert_eq!(read_theia_node(2), "4,5");
        teardown(&r);
    }

    #[test]
    fn dram_freq_trims_to_bare_value() {
        let (r, _guard) = setup("dram_freq");
        w(
            &r,
            "sys/class/devfreq/mtk-dvfsrc-devfreq/cur_freq",
            "4264000\n",
        );
        assert_eq!(get_dram_freq(), "4264000");
        teardown(&r);
    }

    #[test]
    fn first_int_scans_mid_string() {
        assert_eq!(first_int("CPH2493"), 2493);
        assert_eq!(first_int("mt6893"), 6893);
        assert_eq!(first_int("no digits"), 0);
    }

    #[test]
    fn read_fallback_order() {
        let (r, _guard) = setup("fallback");
        w(&r, "proc/oplusVersion/prjName", "20801\n");
        assert_eq!(get_project(), 20801);
        w(&r, "proc/oplusVersion/prjName", "\n");
        io_shim::property_set("ro.product.model", "CPH2493");
        assert_eq!(get_project(), 2493);
        teardown(&r);
    }
}
