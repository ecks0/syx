pub(crate) mod prelude;

use std::iter::once;

use crate::util::stream::prelude::*;
use crate::{Error, Result};

pub(crate) async fn collect<C, S, T, E>(s: S) -> Result<C>
where
    C: Default + Extend<T>,
    S: Stream<Item=std::result::Result<T, E>>,
    E: Into<Error>,
{
    pin_mut!(s);
    let mut c = C::default();
    while let Some(r) = s.next().await {
        let r = r.map_err(Into::into);
        let v = r?;
        let v = once(v);
        c.extend(v);
    }
    Ok(c)
}
