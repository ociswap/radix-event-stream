use crate::decoder::HandlerRegistry;

pub trait DecodableEvent {
    fn name(&self) -> &str;
    fn programmatic_json(&self) -> serde_json::Value;
}

pub trait Transaction {
    fn intent_hash(&self) -> String;
    fn state_version(&self) -> u64;
    fn events(&self) -> Vec<Box<dyn DecodableEvent>>;
}

pub trait TransactionStream {
    fn next(&mut self) -> Option<Vec<Box<dyn Transaction>>>;
}

pub struct EventStreamProcessor<ES>
where
    ES: TransactionStream,
{
    pub event_stream: ES,
    pub decoder_registry: HandlerRegistry,
}

impl<ES> EventStreamProcessor<ES>
where
    ES: TransactionStream,
{
    pub fn new(event_stream: ES, decoder_registry: HandlerRegistry) -> Self {
        EventStreamProcessor {
            event_stream,
            decoder_registry,
        }
    }

    pub fn run(&mut self) {
        loop {
            let resp = match self.event_stream.next() {
                Some(resp) => {
                    if resp.is_empty() {
                        println!(
                            "No more transactions, sleeping for 1 second..."
                        );
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    resp
                }
                None => {
                    println!("Error while getting transactions, trying again");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            };

            resp.iter().for_each(|item| {
                println!("State version: {}", item.state_version());

                let events = item.events();

                events.iter().for_each(|event| {
                    self.decoder_registry.handle(item, event).unwrap();
                })
            });
        }
    }
}
