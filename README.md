# radix-event-stream

### An extensible Rust library to help you identify and process events from the Radix ledger.

## Features

### Extensible:

Easily identify and process custom events by implementing traits on event handlers.

### Data source agnostic:

Pick from one of the providede data sources (Radix Gateway, file) or easily implement your own.

### Easy to use:

Simply pick a transaction source, register your handlers and start the stream.

## Performant:

Leverages the raw performance of Rust 🦀

## Background

[Radix](https://www.radixdlt.com) is a platform for decentralized applications, specifically built for DeFi. Each smart contract, called a component on Radix, can emit events when transactions happen, including custom ones. An event may look somewhat like this:

```Rust
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}
```

These events are recorded inside of transactions and stored on the Radix ledger. This makes it possible to track the state of an application by reading events as they happen, and processing them in some specified way. That's what this library aims to achieve.

Events on Radix have a name, but this name is not unique to the entire network. Consider the case where we have an application which emits events of type `InstantiateEvent`. Another user could create a component which emits events with the exact same name and schema, messing with our data.

That's why we must first `identify` the event, possibly using some application-specific state like package addresses or component addresses. Only when we are sure that the event is ours and we are interested in processing it, should we `process` the event. This can be something like updating a database or notifying other services or users.

## How it works

This library defines an `EventHandler` trait, which requires the `identify` and `process` operations:

```Rust
// Simplified
trait EventHandler {
    fn identify(&self, event: ...) -> Option<...>;
    fn process(&self, event: ..., transaction: ...) -> Result<...>;
    // The handle method combines identify and process
    fn handle(&self, event: ..., transaction: ...) -> Result<...>;
}
```

We can use this trait to implement event handlers, which can then be registered to a `HandlerRegistry`. The `HandlerRegistry` processes events by trying each event handler on the event. If the event is identified by the handler, it will be processed. If it is not identified, nothing happens.

The `HandlerRegistry` is passed to a type implementing the `TransactionStream` trait, which manages the fetching of transactions from a data source and calling `HandlerRegistry::handle()`. This allows us to implement new data sources.

## Usage

Copy-paste your event struct from your smart contract (or import it somehow)

```Rust
#[derive(ScryptoSbor, Debug, EventName)]
pub struct InstantiatePool {
    pool_address: String
}
```

Create a new handler struct with the custom state which we need to identify/process the event.

```Rust
pub struct SimpleEventHandler {
    package_address: String
}
```

Implement `EventHandler` for the `SimpleEventHandler`.

```Rust
impl EventHandler for SimpleEventHandler {
    fn identify(
        &self,
        event: &Box<dyn radix_event_stream::streaming::Event>,
    ) -> Option<Box<dyn Debug>> {
        // Early return if the name doesn't match
        if event.name() != InstantiateEvent::event_name() {
            return None;
        }

        let package_address = match event.emitter() {
            EventEmitterIdentifier::Function {
                package_address, ..
            } => package_address,
            _ => return None,
        };
        if self
            .pool_store
            .borrow()
            .find_by_package_address(package_address)
            .is_none()
        {
            return None;
        }

        let decoded =
            match scrypto_decode::<InstantiateEvent>(&event.binary_sbor_data())
            {
                Ok(decoded) => decoded,
                Err(error) => {
                    log::error!(
                        "Failed to decode InstantiateEvent: {:#?}",
                        error
                    );
                    return None;
                }
            };

        Some(Box::new(decoded))
    }

}
```

Create a new `HandlerRegistry` and register

```Rust
let mut handler_registry = HandlerRegistry::new();
```
