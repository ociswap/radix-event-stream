use std::{future::Future, pin::Pin};

use async_trait::async_trait; // 0.1.71

#[async_trait]
trait Incrementor<S> {
    async fn increment(&self, src: &mut i32, state: S);
}

#[derive(Default)]
struct Inc1;

#[async_trait]
impl<S> Incrementor<S> for Inc1
where
    S: Send + Sync + 'static,
{
    async fn increment(&self, src: &mut i32, state: S) {
        *src = *src + 1
    }
}

#[derive(Default)]
struct Inc2;

#[async_trait]
impl<S> Incrementor<S> for Inc2
where
    S: Send + Sync + 'static,
{
    async fn increment(&self, src: &mut i32, state: S) {
        *src = *src - 2
    }
}

#[async_wrapper_macro::async_wrapper]
async fn incrementer<S>(src: &mut i32, state: S) {
    *src = *src + 1;
}

// fn incrementer<S>(
//     src: &mut i32,
//     state: S,
// ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
//     Box::pin(async {
//         *src = *src + 1;
//     })
// }

#[async_trait]
impl<S, F> Incrementor<S> for F
where
    F: Fn(&mut i32, S) -> (Pin<Box<dyn Future<Output = ()> + Send + '_>>)
        + Send
        + Sync
        + 'static,
    S: Send + Sync + 'static,
{
    async fn increment(&self, src: &mut i32, state: S) {
        self(src, state).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inc1 = Inc1 {};
    let inc2 = Inc2 {};

    // increment_printer(inc1).await;
    // increment_printer(inc2).await;
    let state = ();

    let vec = vec![
        Box::new(inc1) as Box<dyn Incrementor<()>>,
        Box::new(inc2) as Box<dyn Incrementor<()>>,
        Box::new(incrementer) as Box<dyn Incrementor<()>>,
    ];

    let mut src = 0;
    for inc in vec {
        inc.increment(&mut src, ()).await;
    }

    println!("src: {}", src);

    Ok(())
}
