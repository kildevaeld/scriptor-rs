use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;

#[derive(Debug)]
enum Task {
    With(Box<FnOnce() -> Result>),
}

#[derive(Clone)]
struct LocalSpawner {
    send: mpsc::UnboundedSender<Task>,
}

impl LocalSpawner {
    pub async fn new() -> Self {
        let (send, mut recv) = mpsc::unbounded_channel();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some(new_task) = recv.recv().await {
                    // tokio::task::spawn_local(run_task(new_task));
                }
                // If the while loop returns, then all the LocalSpawner
                // objects have have been dropped.
            });

            // This will return once all senders are dropped and all
            // spawned tasks have returned.
            rt.block_on(local);
        });

        Self { send }
    }

    pub fn spawn(&self, task: Task) {
        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.");
    }
}
