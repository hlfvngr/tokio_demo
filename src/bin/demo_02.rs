use std::{future::Future, task::Poll, time::{Duration, Instant}};

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_secs(10);
    let future = Delay{when};
    let res =future.await;
    assert_eq!(res, "done");
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        } 
    }
}

