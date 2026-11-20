// Bounded, read-only I/O. Unit tests never open a host or device node.

fn decode(bytes: Vec<u8>, limit: usize) -> Option<String> {
    if bytes.len() > limit || bytes.contains(&0) {
        return None;
    }
    let text = String::from_utf8(bytes).ok()?;
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return None;
    }
    Some(text.trim_end_matches(['\n', '\r']).to_owned())
}

#[cfg(not(test))]
fn read_open_file(file: std::fs::File, limit: usize) -> Option<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    file.take(limit.checked_add(1)? as u64)
        .read_to_end(&mut bytes)
        .ok()?;
    decode(bytes, limit)
}

#[cfg(not(test))]
pub fn read_file(path: &str, limit: usize) -> Option<String> {
    read_open_file(std::fs::File::open(path).ok()?, limit)
}

#[cfg(all(target_os = "android", not(test)))]
pub fn read_trusted_file(path: &str, limit: usize) -> Option<String> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x20000)
        .open(path)
        .ok()?;
    let metadata = file.metadata().ok()?;
    // O_NOFOLLOW above and fstat below avoid a check/open race on the config.
    if !metadata.is_file() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
        return None;
    }
    read_open_file(file, limit)
}

#[cfg(all(not(target_os = "android"), not(test)))]
pub fn read_trusted_file(_path: &str, _limit: usize) -> Option<String> {
    None
}

#[cfg(all(target_os = "android", not(test)))]
pub fn property_get(key: &str) -> Option<String> {
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    extern "C" {
        fn __system_property_get(name: *const c_char, value: *mut c_char) -> i32;
    }
    let key = CString::new(key).ok()?;
    let mut bytes = [0u8; 92];
    let len = unsafe { __system_property_get(key.as_ptr(), bytes.as_mut_ptr().cast()) };
    if len <= 0 || len as usize >= bytes.len() {
        return None;
    }
    Some(
        CStr::from_bytes_with_nul(&bytes[..=len as usize])
            .ok()?
            .to_str()
            .ok()?
            .to_owned(),
    )
}

#[cfg(all(not(target_os = "android"), not(test)))]
pub fn property_get(_key: &str) -> Option<String> {
    None
}

#[cfg(test)]
thread_local! {
    static FILES: std::cell::RefCell<std::collections::HashMap<String, Vec<u8>>> = Default::default();
    static PROPS: std::cell::RefCell<std::collections::HashMap<String, String>> = Default::default();
    static READS: std::cell::RefCell<Vec<String>> = Default::default();
}

#[cfg(test)]
pub fn read_file(path: &str, limit: usize) -> Option<String> {
    READS.with(|r| r.borrow_mut().push(path.to_owned()));
    decode(FILES.with(|f| f.borrow().get(path).cloned())?, limit)
}

#[cfg(test)]
pub fn read_trusted_file(path: &str, limit: usize) -> Option<String> {
    read_file(path, limit)
}

#[cfg(test)]
pub fn property_get(key: &str) -> Option<String> {
    READS.with(|r| r.borrow_mut().push(key.to_owned()));
    PROPS.with(|p| p.borrow().get(key).cloned())
}

#[cfg(test)]
pub fn mock_file(path: &str, text: &str) {
    FILES.with(|f| {
        f.borrow_mut()
            .insert(path.to_owned(), text.as_bytes().to_vec())
    });
}

#[cfg(test)]
pub fn mock_property(key: &str, text: &str) {
    PROPS.with(|p| p.borrow_mut().insert(key.to_owned(), text.to_owned()));
}

#[cfg(test)]
pub fn mock_reads() -> Vec<String> {
    READS.with(|r| r.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversize_binary_and_control_data() {
        assert!(decode(vec![b'x'; 5], 4).is_none());
        assert!(decode(vec![0xff], 4).is_none());
        assert!(decode(vec![b'x', 0], 4).is_none());
        assert!(decode(vec![0x1b], 4).is_none());
        assert_eq!(decode(b"123\n".to_vec(), 4).as_deref(), Some("123"));
    }

    #[test]
    fn mocks_are_thread_local_and_do_not_open_missing_nodes() {
        mock_file("/mock/node", "12");
        assert_eq!(read_file("/mock/node", 4).as_deref(), Some("12"));
        assert!(std::thread::spawn(|| read_file("/mock/node", 4))
            .join()
            .unwrap()
            .is_none());
        assert!(read_file("/proc/not-a-real-test-source", 4).is_none());
    }
}
