use std::fmt::Debug;
use std::sync::Arc;

use futures::Future;
use tokio::sync::Mutex;

use crate::Result;

#[derive(Clone, Debug)]
pub(crate) struct Cell<T>
where
    T: Clone + Debug + Send + 'static,
{
    cell: Arc<Mutex<Option<T>>>,
}

impl<T> Cell<T>
where
    T: Clone + Debug + Send + 'static,
{
    pub(crate) async fn get_or_load<F>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let mut value = self.cell.lock().await;
        if let Some(v) = value.clone() {
            #[cfg(feature = "logging")]
            log::trace!("OK cache HIT {:?}", v);
            Ok(v)
        } else {
            let v = f.await;
            #[cfg(feature = "logging")]
            match &v {
                Ok(v) => log::trace!("OK cache MISS {:?}", v),
                Err(e) => log::trace!("ERR cache {}", e.to_string()),
            }
            let v = v?;
            value.replace(v.clone());
            Ok(v)
        }
    }

    pub(crate) async fn clear(&self) {
        self.cell.lock().await.take();
    }

    pub(crate) async fn clear_if_ok<F, R>(&self, f: F) -> Result<R>
    where
        F: Future<Output = Result<R>>,
        R: Send + 'static,
    {
        let mut value = self.cell.lock().await;
        let r = f.await;
        if r.is_ok() {
            value.take();
        }
        r
    }
}

impl<T> Default for Cell<T>
where
    T: Clone + Debug + Send + 'static,
{
    fn default() -> Self {
        let cell = Arc::new(Mutex::new(None));
        Self { cell }
    }
}
