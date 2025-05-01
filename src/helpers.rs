use crate::{
    CallResponse,
    AlkaneTransferParcel,
    Result,
    anyhow,
};

/// # Convert u128 vector to String
///
/// Converts a slice of u128 values to a UTF-8 string.
/// Used primarily for decoding string parameters from opcode inputs.
///
/// # Parameters
/// * `values` - Slice of u128 values to convert
///
/// # Returns
/// * `Result<String>` - Converted string or error
pub fn u128_vec_to_string(values: &[u128]) -> Result<String> {
    if values.is_empty() {
        return Ok(String::new());
    }
    
    // Convert each u128 to bytes and join
    let mut bytes = Vec::new();
    for &value in values {
        let value_bytes = value.to_le_bytes();
        bytes.extend_from_slice(&value_bytes);
    }
    
    // Remove trailing zeros
    while let Some(&0) = bytes.last() {
        bytes.pop();
    }
    
    // Convert bytes to string
    String::from_utf8(bytes)
        .map_err(|e| anyhow!("Failed to convert bytes to UTF-8 string: {}", e))
}

/// # Convert u128 to u64
///
/// Safely converts a u128 value to u64, checking for overflow.
///
/// # Parameters
/// * `value` - The u128 value to convert
///
/// # Returns
/// * `u64` - The converted value, or u64::MAX if overflow occurs
pub fn u128_to_u64(value: u128) -> u64 {
    if value > u64::MAX as u128 {
        u64::MAX
    } else {
        value as u64
    }
}

/// # Convert u128 to u16
///
/// Safely converts a u128 value to u16, checking for overflow.
///
/// # Parameters
/// * `value` - The u128 value to convert
///
/// # Returns
/// * `u16` - The converted value, or u16::MAX if overflow occurs
pub fn u128_to_u16(value: u128) -> u16 {
    if value > u16::MAX as u128 {
        u16::MAX
    } else {
        value as u16
    }
}

/// # Convert String to Option<String>
///
/// Converts a String to Option<String>, treating empty strings as None.
///
/// # Parameters
/// * `value` - The string to convert
///
/// # Returns
/// * `Option<String>` - Some(value) if non-empty, None if empty
pub fn string_to_option_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

/// # Create CallResponse with log message
///
/// Creates a CallResponse with a log message in the data field.
///
/// # Parameters
/// * `message` - The log message to include
///
/// # Returns
/// * `CallResponse` - A response with the message as data
pub fn call_response_with_log(message: String) -> CallResponse {
    CallResponse {
        alkanes: AlkaneTransferParcel::default(),
        data: message.as_bytes().to_vec(),
    }
}

/// # Truncate String if too long
///
/// Truncates a string if it's longer than the specified length.
///
/// # Parameters
/// * `s` - String to potentially truncate
/// * `max_len` - Maximum length
///
/// # Returns
/// * `String` - Truncated string if necessary, or original string
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len.saturating_sub(3)])
    }
}

/// # Convert u64 to u128 safely
///
/// Safely converts a u64 to u128.
///
/// # Parameters
/// * `value` - The u64 value to convert
///
/// # Returns
/// * `u128` - The converted value
pub fn u64_to_u128(value: u64) -> u128 {
    value as u128
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_u128_vec_to_string() {
        // ASCII text "hello" as bytes packed into u128
        let values = vec![0x6f6c6c6568]; // "hello" in little endian
        let result = u128_vec_to_string(&values).unwrap();
        assert_eq!(result, "hello");
    }
    
    #[test]
    fn test_u128_to_u64() {
        assert_eq!(u128_to_u64(100), 100);
        assert_eq!(u128_to_u64(u64::MAX as u128), u64::MAX);
        assert_eq!(u128_to_u64(u64::MAX as u128 + 1), u64::MAX);
    }
    
    #[test]
    fn test_u128_to_u16() {
        assert_eq!(u128_to_u16(100), 100);
        assert_eq!(u128_to_u16(u16::MAX as u128), u16::MAX);
        assert_eq!(u128_to_u16(u16::MAX as u128 + 1), u16::MAX);
    }
    
    #[test]
    fn test_string_to_option_string() {
        assert_eq!(string_to_option_string("hello".to_string()), Some("hello".to_string()));
        assert_eq!(string_to_option_string("".to_string()), None);
        assert_eq!(string_to_option_string(" ".to_string()), None);
    }
    
    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 5), "he...");
    }
}
