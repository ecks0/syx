use std::future::Future;
use std::sync::Arc;

use parking_lot::FairMutex;
use tokio::task::spawn_blocking;

use crate::Result;

#[derive(Clone, Debug)]
pub(crate) struct Cell<T>
where
    T: Clone + Send + 'static,
{
    cell: Arc<FairMutex<Option<T>>>,
}

impl<T> Cell<T>
where
    T: Clone + Send + 'static,
{
    async fn set(&self, v: T) {
        let cell = Arc::clone(&self.cell);
        spawn_blocking(move || cell.lock().replace(v))
            .await
            .unwrap();
    }

    async fn get(&self) -> Option<T> {
        let cell = Arc::clone(&self.cell);
        spawn_blocking(move || cell.lock().as_ref().cloned())
            .await
            .unwrap()
    }

    pub(crate) async fn get_or_load<F>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        if let Some(v) = self.get().await {
            Ok(v)
        } else {
            match f.await {
                Ok(v) => {
                    self.set(v.clone()).await;
                    Ok(v)
                },
                Err(e) => Err(e),
            }
        }
    }

    pub(crate) async fn clear(&self) {
        let cell = Arc::clone(&self.cell);
        spawn_blocking(move || cell.lock().take()).await.unwrap();
    }

    pub(crate) async fn clear_if_ok<F, R>(&self, f: F) -> Result<R>
    where
        F: Future<Output = Result<R>>,
        R: Send + 'static,
    {
        let r = f.await;
        if r.is_ok() {
            self.clear().await;
        }
        r
    }
}

impl<T> Default for Cell<T>
where
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        let cell = Arc::new(FairMutex::new(None));
        Self { cell }
    }
}
