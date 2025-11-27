use std::fmt;
use std::ops::Deref;

#[cfg(unix)]
use libc;

/// A buffer that zeroizes its content on drop and prevents swapping (on Unix).
/// Used for storing sensitive data like private keys and mnemonics.
#[derive(Clone)]
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        let buffer = Self { data };
        buffer.lock_memory();
        buffer
    }

    pub fn from_string(s: String) -> Self {
        Self::new(s.into_bytes())
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    #[cfg(unix)]
    fn lock_memory(&self) {
        if self.data.is_empty() {
            return;
        }

        unsafe {
            let ptr = self.data.as_ptr() as *const libc::c_void;
            let len = self.data.len();
            libc::mlock(ptr, len);
        }
    }

    #[cfg(not(unix))]
    fn lock_memory(&self) {
        // No-op on non-Unix systems
    }

    #[cfg(unix)]
    fn unlock_memory(&self) {
        if self.data.is_empty() {
            return;
        }

        unsafe {
            let ptr = self.data.as_ptr() as *const libc::c_void;
            let len = self.data.len();
            libc::munlock(ptr, len);
        }
    }

    #[cfg(not(unix))]
    fn unlock_memory(&self) {
        // No-op on non-Unix systems
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        unsafe {
            for byte in self.data.iter_mut() {
                std::ptr::write_volatile(byte, 0x00);
            }
        }
        self.unlock_memory();
    }
}

impl Deref for SecureBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<[u8]> for SecureBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl fmt::Debug for SecureBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureBuffer(***REDACTED***)")
    }
}

impl From<Vec<u8>> for SecureBuffer {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<String> for SecureBuffer {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for SecureBuffer {
    fn from(s: &str) -> Self {
        Self::from_string(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_buffer_creation() {
        let data = vec![1, 2, 3];
        let buffer = SecureBuffer::new(data.clone());
        assert_eq!(*buffer, data[..]);
    }

    #[test]
    fn test_secure_buffer_from_string() {
        let s = "secret";
        let buffer = SecureBuffer::from(s);
        assert_eq!(buffer.as_str().unwrap(), s);
    }

    #[test]
    fn test_debug_redaction() {
        let buffer = SecureBuffer::from("secret");
        assert_eq!(format!("{:?}", buffer), "SecureBuffer(***REDACTED***)");
    }
}
