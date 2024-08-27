use std::{future::Future, pin::Pin, task::Poll, thread, time::{Duration, Instant}};

use futures::task::{self};

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
            // cx.waker().wake_by_ref(); 在这个模型里面，这句话没有起作用
            Poll::Pending
        } 
    }
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

struct MiniTokio {
    tasks: Vec<Task>,
}

impl MiniTokio {

    fn new() -> Self {
        MiniTokio {
            tasks: Vec::new(),
        }
    }

    fn spawn(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        self.tasks.push(Box::pin(future));
    }

    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = task::Context::from_waker(&waker);
        while let Some(mut future) = self.tasks.pop() {
            println!("mini tokio run: {}", thread::current().name().unwrap());
            if future.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push(future);
            }
        }
    }
}
