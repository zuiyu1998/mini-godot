use futures::executor::ThreadPool;
use mini_core::uuid::Uuid;
use parking_lot::Mutex;
use std::{
    any::Any,
    future::Future,
    sync::mpsc::{self, Receiver, Sender},
};

pub struct TaskPool {
    thread_pool: ThreadPool,
    sender: Sender<TaskResult>,
    receiver: Mutex<Receiver<TaskResult>>,
}

impl TaskPool {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            thread_pool: ThreadPool::new().unwrap(),
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    //执行task
    pub fn spawn_task<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.thread_pool.spawn_ok(future);
    }

    //提供给资源加载器的接口，异步加载资源
    #[inline]
    pub fn spawn_with_result<F, T>(&self, future: F) -> Uuid
    where
        F: AsyncTask<T>,
        T: AsyncTaskResult,
    {
        let id = Uuid::new_v4();
        let sender = self.sender.clone();
        self.spawn_task(async move {
            let result = future.await;
            sender
                .send(TaskResult {
                    id,
                    payload: Box::new(result),
                })
                .unwrap();
        });
        id
    }

    #[inline]
    pub fn next_task_result(&self) -> Option<TaskResult> {
        self.receiver.lock().try_recv().ok()
    }
}

pub struct TaskResult {
    pub id: Uuid,
    pub payload: Box<dyn AsyncTaskResult>,
}

pub trait AsyncTask<R: AsyncTaskResult>: Future<Output = R> + Send + 'static {}

impl<T, R: AsyncTaskResult> AsyncTask<R> for T where T: Future<Output = R> + Send + 'static {}

pub trait AsyncTaskResult: Any + Send + 'static {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> AsyncTaskResult for T
where
    T: Any + Send + 'static,
{
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl dyn AsyncTaskResult {
    pub fn downcast<T: AsyncTaskResult>(self: Box<Self>) -> Result<Box<T>, Box<dyn Any>> {
        self.into_any().downcast()
    }
}
