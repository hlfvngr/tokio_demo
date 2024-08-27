use std::{future::Future, pin::Pin, sync::{mpsc, Arc, Mutex}, task::{Context, Poll}, thread, time::{Duration, Instant}};

use futures::task::{self, ArcWake};

#[tokio::main]
async fn main() {
    let mut mini_tokio = MiniTokio::new();
    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_secs(10);
        let future = Delay{when};
        let res = future.await;
        println!("{}", res);
    });

    mini_tokio.run();
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("delay poll: {}", thread::current().name().unwrap());
        if Instant::now() >= self.when {
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        } 
    }
}

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {

    fn new() -> Self {
        let (sender, scheduled) = mpsc::channel();
        MiniTokio {
            sender, scheduled
        }
    }

    fn spawn(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        Task::spawn(future, &mut self.sender)
    }

    fn run(&mut self) {
        while let Ok(t) = self.scheduled.recv() {
            t.poll();
        }
    }
}


struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    poll: Poll<()>,
}

impl TaskFuture {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }
}

impl TaskFuture {

    pub fn poll(mut self: &mut Self, cx: &mut std::task::Context<'_>) {
        if self.poll.is_pending() {
            let mut future = self.future.as_mut();
            self.poll = future.poll(cx);
        }
    }
}
struct Task {
    task_future: Mutex<TaskFuture>,
    executer: mpsc::Sender<Arc<Task>>,
}

impl ArcWake for Task {

    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.scheduled();
    }
}

impl Task {  

    fn spawn(future: impl Future<Output = ()> + Send + 'static, executor: &mut mpsc::Sender<Arc<Task>>) {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executer: executor.clone(),
        });
        let _ = executor.send(task);
    }

    fn scheduled(self: &Arc<Self>) {
        self.executer.send(self.clone());
    }

    fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);
        let mut task_future = self.task_future.try_lock().unwrap();
        task_future.poll(&mut cx);
    }
}

