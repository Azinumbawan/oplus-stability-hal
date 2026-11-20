pub const DESCRIPTOR: &str = "vendor.oplus.hardware.stability.oplus_project.IOplusProject";
pub const INTERFACE_VERSION: i32 = 3;
pub const INTERFACE_HASH: &str = "3e738561c0bfdc622a1c84face254caf0c887c8f";
pub const METHOD_COUNT: u32 = 35;
pub const MAX_STRING_BYTES: usize = 65536;
pub const MAX_INT_ARRAY: usize = 4096;
pub const STATUS_BAD_VALUE: i32 = -22;
pub const STATUS_UNKNOWN_TRANSACTION: i32 = -74;
pub const STATUS_UNEXPECTED_NULL: i32 = i32::MIN + 8;

pub fn privileged(uid: u32) -> bool {
    matches!(uid, 0 | 1000)
}

pub fn consumed(position: i32, size: i32) -> Result<(), i32> {
    if position < 0 || !(0..=1 << 20).contains(&size) || position != size {
        Err(STATUS_BAD_VALUE)
    } else {
        Ok(())
    }
}

pub fn checked_length(length: i32, maximum: usize, string: bool) -> Result<Option<usize>, i32> {
    if length == -1 {
        return Ok(None);
    }
    if length < 0 || (string && length == 0) || length as usize > maximum {
        return Err(STATUS_BAD_VALUE);
    }
    Ok(Some(length as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donor_v3_tail_and_metadata_are_coherent() {
        let aidl = include_str!(
            "../../aidl/vendor/oplus/hardware/stability/oplus_project/IOplusProject.aidl"
        );
        let methods: Vec<_> = aidl
            .lines()
            .filter(|s| s.contains('(') && s.trim_end().ends_with(';'))
            .collect();
        assert_eq!(methods.len(), METHOD_COUNT as usize);
        assert!(methods[33].contains("String read_op1_record_info()"));
        assert!(methods[34].contains("boolean write_op1_record_info(in String op1_logs_data)"));
        assert_eq!(INTERFACE_VERSION, 3);
        assert_eq!(INTERFACE_HASH, "3e738561c0bfdc622a1c84face254caf0c887c8f");
        assert_eq!(STATUS_UNKNOWN_TRANSACTION, -74);
    }

    #[test]
    fn lengths_nulls_and_trailing_data_are_distinct() {
        assert_eq!(checked_length(-1, 4, true), Ok(None));
        assert_eq!(checked_length(-2, 4, true), Err(STATUS_BAD_VALUE));
        assert_eq!(checked_length(0, 4, true), Err(STATUS_BAD_VALUE));
        assert_eq!(checked_length(0, 4, false), Ok(Some(0)));
        assert_eq!(checked_length(5, 4, false), Err(STATUS_BAD_VALUE));
        assert_eq!(consumed(4, 4), Ok(()));
        assert_eq!(consumed(4, 8), Err(STATUS_BAD_VALUE));
        assert_eq!(consumed(-1, -1), Err(STATUS_BAD_VALUE));
        assert!(!privileged(u32::MAX));
    }
}
