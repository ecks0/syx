#[cfg(feature = "sync")]
pub(crate) use sync::Cached;
#[cfg(not(feature = "sync"))]
pub(crate) use unsync::Cached;

#[cfg(feature = "sync")]
mod sync {
    use std::future::Future;
    use std::sync::Arc;

    use parking_lot::FairMutex;
    use tokio::task::spawn_blocking;

    use crate::Result;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct Cached<T>
    where
        T: Clone + Send + 'static,
    {
        cell: Arc<FairMutex<Option<T>>>,
    }

    impl<T> Cached<T>
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

        pub(crate) async fn get_or<F>(&self, f: F) -> Result<T>
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

        pub(crate) async fn clear_if<F, R>(&self, f: F) -> Result<R>
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
}

#[cfg(not(feature = "sync"))]
mod unsync {
    use std::cell::RefCell;
    use std::future::Future;
    use std::sync::Arc;

    use crate::Result;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct Cached<T>
    where
        T: Clone + Send + 'static,
    {
        cell: Arc<RefCell<Option<T>>>,
    }

    impl<T> Cached<T>
    where
        T: Clone + Send + 'static,
    {
        async fn set(&self, v: T) {
            self.cell.borrow_mut().replace(v);
        }

        async fn get(&self) -> Option<T> {
            self.cell.borrow().as_ref().cloned()
        }

        pub(crate) async fn get_or<F>(&self, f: F) -> Result<T>
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
            self.cell.borrow_mut().take();
        }

        pub(crate) async fn clear_if<F, R>(&self, f: F) -> Result<R>
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
}
