# radix-event-stream

### An extensible Rust library to help you identify and process events from the Radix ledger.

## Features

### ✅ Extensible:

Easily identify and process custom events by implementing event handlers.

### ✅ Data source agnostic:

Pick from one of the provided data sources (Radix Gateway, file) or easily implement your own.

### ✅ Easy to use:

Simply pick a transaction source, register your handlers and start the stream.

### ✅ Performant:

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

These events are recorded inside of transactions in-sequence and stored on the Radix ledger. This allows developers to track the state of an application by reading events as they happen, and processing them in some specified way. That's what this library aims to achieve.

Events on Radix have an **emitter**. This is the on-ledger entity which emitted the event. Events also have a **name**, which is the name of the event and its data type.

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

We can use this trait to implement event handlers, which can then be registered to a `HandlerRegistry`. The `HandlerRegistry` processes events by trying each event handler on the event. If the event is identified by the handler, it will be processed. If it is not identified, it is ignored.

The `HandlerRegistry` is passed to a type implementing the `TransactionStream` trait, which manages the fetching of transactions from a data source and calling `HandlerRegistry::handle()`. This allows us to implement new custom data sources.

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
        event: &Event,
    ) -> Option<Box<dyn Debug>> {
        // Early return if the name doesn't match.
        // We can use the derived EventName trait
        // to minimize typo errors.
        if event.name() != SimpleEvent::event_name() {
            return None;
        }

        // Check whether the emitter of this event is our package
        let package_address = match event.emitter() {
            EventEmitterIdentifier::Function {
                package_address, ..
            } => package_address,
            _ => return None,
        };
        if self.package_address != package_address
        {
            return None;
        }

        // Decode and return the decoded struct.
        // This allows the framework to log it.
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
    fn process(
        &self,
        event: &Event,
        _: &Transaction,
    ) -> Result<(), Box<dyn Error>> {
        // Decode the event again using scrypto_decode
        let decoded: InstantiateEvent =
            match scrypto_decode::<InstantiateEvent>(&event.binary_sbor_data())
            {
                Ok(decoded) => decoded,
                Err(error) => {
                    log::error!(
                        "Failed to decode InstantiateEvent: {:#?}",
                        error
                    );
                    return Err("Failed to decode InstantiateEvent".into());
                }
            };
        // Do any kind of processing.
        Ok(())
    }
}
```

Create a new `HandlerRegistry` and register the handler.

```Rust
let package_address = "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx".to_string()
let mut handler_registry = HandlerRegistry::new();
handler_registry.add_handler(SimpleEventHandler {
    package_address,
});
```

Initialize some type implementing `TransactionStream`. There is a Gateway stream available out of the box. Then, start the `TransactionStreamProcessor`. That's it!

```Rust
let stream =
    GatewayTransactionStream::new(
        8000000, // State version to start at
        100 // Items to fetch per page
        "https://mainnet.radixdlt.com".to_string(),
    );

// Start with parameters.
TransactionStreamProcessor::run_with(
    stream,
    handler_registry,
    // Interval for logging current state version
    Some(std::time::Duration::from_secs(1)),
);
```
