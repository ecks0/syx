use std::sync::Arc;

use futures::Future;
use tokio::sync::Mutex;

use crate::Result;

#[derive(Clone, Debug)]
pub(crate) struct Cell<T>
where
    T: Clone + Send + 'static,
{
    cell: Arc<Mutex<Option<T>>>,
}

impl<T> Cell<T>
where
    T: Clone + Send + 'static,
{
    pub(crate) async fn get_or_load<F>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let mut value = self.cell.lock().await;
        if let Some(v) = value.clone() {
            Ok(v)
        } else {
            let v = f.await?;
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
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        let cell = Arc::new(Mutex::new(None));
        Self { cell }
    }
}
