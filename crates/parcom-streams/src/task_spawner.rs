pub mod thread_spawner;

use std::future::IntoFuture;

pub use thread_spawner::ThreadSpawner;

pub trait TaskSpawner {
    type Error;
    type Handle<T: Send>: IntoFuture<Output = Result<T, Self::Error>>;
    fn spawn<T, F>(&self, task: F) -> Self::Handle<T>
    where
        T: 'static + Send,
        F: 'static + Send + IntoFuture<Output = T>;
}
