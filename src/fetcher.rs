pub struct Fetcher<T> {
    rx: tokio::sync::oneshot::Receiver<Result<T, anyhow::Error>>,
}

impl<T> Fetcher<T> {
    pub fn new<F>(task: F) -> Self
    where
        F: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let task_result = task.await;
            tx.send(task_result).ok()
        });
        Self { rx }
    }

    pub fn try_get(&mut self) -> Option<T> {
        self.rx.try_recv().ok().and_then(|r| r.ok())
    }
}

pub enum ResourceState<T> {
    Loading,
    Loaded(T),
    Failed,
}

impl<T> ResourceState<T> {
    pub fn get(&self) -> Option<&T> {
        match self {
            ResourceState::Loaded(data) => Some(data),
            _ => None,
        }
    }
}
