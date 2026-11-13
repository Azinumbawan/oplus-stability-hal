// File and property plumbing for the OplusProject HAL.
//
// Node reads go through std::fs with an optional test root so the whole
// adapter set is unit-testable on the host against fixture files. Properties
// mirror bionic's __system_property_get on Android and an in-memory store off
// target.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static TEST_ROOT: Mutex<Option<String>> = Mutex::new(None);

/// Prepends the test root when set, so host tests can point the adapter at a
/// fixtures directory instead of the real /proc and /sys. Every test holds
/// TESTS while it runs, so the root is only ever seen by one test.
fn resolve(path: &Path) -> PathBuf {
    if path.is_absolute() {
        let root = TEST_ROOT.lock().unwrap().clone();
        if let Some(root) = root {
            let stripped = path
                .strip_prefix("/")
                .expect("absolute path strips cleanly");
            return Path::new(&root).join(stripped);
        }
    }
    path.to_path_buf()
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Option<String> {
    let out = fs::read_to_string(resolve(path.as_ref())).ok()?;
    Some(out.trim_end_matches(['\n', '\r']).to_string())
}

pub fn write_file<P: AsRef<Path>>(path: P, value: &str) -> bool {
    match fs::write(resolve(path.as_ref()), value) {
        Ok(()) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
pub fn set_test_root(root: &str) {
    *TEST_ROOT.lock().unwrap() = Some(root.to_string());
}

#[cfg(test)]
pub fn clear_test_root() {
    *TEST_ROOT.lock().unwrap() = None;
}

/// Serializes the test module: the root override is process-global, so the
/// fixture-using tests must not overlap.
#[cfg(test)]
static TESTS: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TESTS.lock().unwrap()
}

#[cfg(target_os = "android")]
pub fn property_get(key: &str) -> Option<String> {
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    extern "C" {
        fn __system_property_get(name: *const c_char, value: *mut c_char) -> i32;
    }

    const PROP_VALUE_MAX: usize = 92;

    let name = CString::new(key).ok()?;
    let mut buf = vec![0u8; PROP_VALUE_MAX];
    // SAFETY: name is a valid NUL-terminated C string and buf is
    // PROP_VALUE_MAX bytes as bionic's contract requires.
    let n = unsafe { __system_property_get(name.as_ptr(), buf.as_mut_ptr() as *mut c_char) };
    if n <= 0 {
        return None;
    }
    let value = CStr::from_bytes_with_nul(&buf[..=n as usize]).ok()?;
    Some(value.to_string_lossy().into_owned())
}

#[cfg(not(target_os = "android"))]
pub fn property_get(key: &str) -> Option<String> {
    with_props(|p| p.get(key).cloned())
}

#[cfg(not(target_os = "android"))]
fn with_props<R>(f: impl FnOnce(&mut std::collections::HashMap<String, String>) -> R) -> R {
    use std::sync::Mutex;
    static PROPS: Mutex<Option<std::collections::HashMap<String, String>>> = Mutex::new(None);
    let mut store = PROPS.lock().unwrap();
    if store.is_none() {
        *store = Some(std::collections::HashMap::new());
    }
    f(store.as_mut().unwrap())
}

#[cfg(not(target_os = "android"))]
pub fn property_set(key: &str, value: &str) -> bool {
    with_props(|p| {
        p.insert(key.to_string(), value.to_string());
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_roundtrip() {
        let _g = test_lock();
        let dir = std::env::temp_dir().join("oplus_project_io_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("node");
        assert!(write_file(&path, "12345"));
        assert_eq!(read_file(&path).unwrap(), "12345");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_file_is_none() {
        assert!(read_file("/this/does/not/exist/oplus_project").is_none());
    }

    #[test]
    fn test_root_override() {
        let _g = test_lock();
        let root = "/tmp/opencode/stab_override_root";
        std::fs::create_dir_all(format!("{}/proc", root)).unwrap();
        set_test_root(root);
        assert!(write_file("/proc/override_node", "99"));
        assert_eq!(read_file("/proc/override_node").unwrap(), "99");
        clear_test_root();
        assert!(read_file("/proc/override_node").is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(not(target_os = "android"))]
    #[test]
    fn host_property_roundtrip() {
        assert!(property_get("oplus_project.test.key").is_none());
        assert!(property_set("oplus_project.test.key", "value1"));
        assert_eq!(property_get("oplus_project.test.key").unwrap(), "value1");
    }
}
