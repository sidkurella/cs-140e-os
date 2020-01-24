/// Gets a detailed string description for the given error number.
pub fn error_string(_errno: i32) -> String {
    "operation successful".to_string()
}

/// Returns the platform-specific value of errno
pub fn errno() -> i32 {
    0
}
