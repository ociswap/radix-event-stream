#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let handlers: Vec<Box<dyn Handler>> =
        vec![force_boxed!(handle_event), force_boxed!(handle_event)];
    let mut event = "event".to_string();
    for handler in handlers {
        handler.handle(&mut event).await;
    }
}

#[async_trait::async_trait]
trait Handler {
    async fn handle(&self, event: &mut String);
}

#[async_trait::async_trait]
impl<F> Handler for F
where
    F: Fn(&mut String) + Send + Sync,
{
    async fn handle(&self, event: &mut String) {
        self(event);
    }
}

async fn handle_event(event: &mut String) {
    event.push_str(" handled");
    println!("Handling event: {}", event);
    // Do something with the event
}

macro_rules! force_boxed {
    ($inc:expr) => {{
        // I think the error message referred to here is spurious, but why take a chance?
        fn rustc_complains_if_this_name_conflicts_with_the_environment_even_though_its_probably_fine(src: u32) -> Pin<Box<dyn Future<Output = u32>>> {
            Box::pin($inc(src))
        }
        rustc_complains_if_this_name_conflicts_with_the_environment_even_though_its_probably_fine
    }}
}
