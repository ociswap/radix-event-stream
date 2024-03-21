use crate::decoder::HandlerRegistry;

pub trait DecodableEvent {
    fn name(&self) -> &'static str;
    fn programmatic_json(&self) -> serde_json::Value;
}

pub trait Transaction {
    fn intent_hash(&self) -> String;
    fn state_version(&self) -> u64;
    fn events<E: DecodableEvent>(&self) -> Vec<E>;
}

pub trait EventStream{
    fn next<T, E>(&mut self)
    where
        T: Transaction<E>,
        E: DecodableEvent;
     -> Option<Vec<T>>;
}

pub struct EventStreamProcessor<ES, T, E>
where
    ES: EventStream<T, E>,
    T: Transaction<E>,
    E: DecodableEvent,
{
    pub event_stream: ES,
    pub decoder_registry: HandlerRegistry<E, T>,
    _marker: std::marker::PhantomData<T>,
    _marker2: std::marker::PhantomData<E>,
}

impl<ES, T, E> EventStreamProcessor<ES, T, E>
where
    ES: EventStream<T, E>,
    T: Transaction<E>,
    E: DecodableEvent,
{
    pub fn new(
        event_stream: ES,
        decoder_registry: HandlerRegistry<E, T>,
    ) -> Self {
        EventStreamProcessor {
            event_stream,
            decoder_registry,
            _marker: std::marker::PhantomData,
            _marker2: std::marker::PhantomData,
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
