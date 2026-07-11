//! Small deterministic host utilities.

use crate::{host, Error, Result};

pub fn sleep(milliseconds: u32) -> Result<()> {
    host::sleep_millis(milliseconds).map_err(Error::new)
}
