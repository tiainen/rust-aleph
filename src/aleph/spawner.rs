use aleph_bft::{SpawnHandle, TaskHandle};
use futures::Future;
use log::debug;

#[derive(Clone)]
pub struct Spawner;

impl SpawnHandle for Spawner {

    fn spawn(&self, _: &str, task: impl Future<Output = ()> + Send + 'static) {
        debug!("Spawner::spawn");
        tokio::spawn(task);
    }

    fn spawn_essential(
        &self,
        _: &str,
        task: impl Future<Output = ()> + Send + 'static,
    ) -> TaskHandle {
        debug!("Spawner::spawn_essential");
        Box::pin(async move { tokio::spawn(task).await.map_err(|_| ()) })
    }
}
