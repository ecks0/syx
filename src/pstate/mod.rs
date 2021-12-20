pub mod policy;
pub mod system;

use crate::Result;

pub async fn available() -> Result<bool> {
    Ok(system::path::status().is_file())
}
