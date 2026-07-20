//! Small host utilities available without ambient WASI access.

use crate::{host, Error, Result};

pub fn sleep(milliseconds: u32) -> Result<()> {
    host::sleep_millis(milliseconds).map_err(Error::new)
}

/// Current host wall-clock time in Unix milliseconds.
pub fn now_millis() -> i64 {
    host::now_millis()
}

/// Cryptographically secure random bytes supplied by the host.
pub fn random_bytes(length: u32) -> Result<Vec<u8>> {
    host::random_bytes(length).map_err(Error::new)
}
