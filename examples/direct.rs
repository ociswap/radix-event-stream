#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let handlers: Vec<fn(&mut String)> = vec![handle_event, handle_event];
    let mut event = "event".to_string();
    for handler in handlers {
        handler(&mut event);
    }
}

fn handle_event(event: &mut String) {
    event.push_str(" handled");
    println!("Handling event: {}", event);
    // Do something with the event
}
