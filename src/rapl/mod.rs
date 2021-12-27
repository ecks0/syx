pub mod constraint;
pub(crate) mod path;
pub mod zone;

use crate::Result;

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}
