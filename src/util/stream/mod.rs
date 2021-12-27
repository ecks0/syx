pub(crate) mod prelude;

use std::iter::once;

use crate::util::stream::prelude::*;
use crate::Result;

pub(crate) async fn collect<C, S, T>(s: S) -> Result<C>
where
    C: Default + Extend<T>,
    S: Stream<Item=Result<T>>,
{
    pin_mut!(s);
    let mut c = C::default();
    while let Some(i) = s.next().await {
        let i = i?;
        let i = once(i);
        c.extend(i);
    }
    Ok(c)
}
