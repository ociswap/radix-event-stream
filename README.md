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

Events on Radix are encoded as SBOR (Scrypto Binary-friendly Object Representation), a custom format supporting binary and json representations. Radix provides Rust crates which can decode SBOR directly into the events as we defined them in a blueprint.

Each event has an **emitter**. This is the on-ledger entity which emitted the event. Events also have a **name**, which is the name of the event and its data type on ledger. These two identifiers are enough to map incoming events to handlers.

## General usage.

### Step 1: copy over event definitions.

```Rust
#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    context_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}
```

Above we see an event definition used in one of Ociswap's Basic pools. It derives at the very least `radix_engine_common::ScryptoSbor`, which is needed to decode it from binary SBOR. Copy this over to your project.

### Step 2: Define an application state.

It should at least implement Clone. If you need to share this data with other pieces of code you may choose to store items wrapped in Rc, RefCell, Arc, RefCell, etc.

```rust
#[derive(Clone)]
pub struct AppState {
    instantiate_events_seen: u64
}
```

### Step 3: implement handlers.

Write a handler which conforms to the predefined handler signature. It must take in the `EventHandlerContext`, which stores things like the current Radix transaction and the application state defined in step 1. I also takes the decoded event type as we copied over from our blueprint in step 1.

The `EventHandler` trait actually defines handlers to take in a `Vec<u8>` instead of the event type itself, but the `#[event_handler]` macro expands the function to handle decoding of the event for you.

```Rust
#[event_handler]
pub fn handle_instantiate_event(
    context: EventHandlerContext<AppState>,
    event: InstantiateEvent,
) -> Result<(), EventHandlerError> {
    info!(
        "Handling the {}th instantiate event: {:#?}",
        context.app_state.number, event
    );
    context.app_state.number += 1;
    Ok(())
}
```

This handler counts the amount of `InstantiateEvents` seen inside an app state variable as we go through the ledger.

### Step 4: Register handlers.

Create a handler registry:

```Rust
let mut handler_registry: HandlerRegistry<AppState> =
    HandlerRegistry::new();
```

Add any handlers to the registry, identified by emitters and event names. In this case, we would like to handle `InstantiateEvent`s emitted by Ociswap's Basic pool package address.

```rust
// Add the instantiate event handler to the registry
handler_registry.add_handler(
    "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
    "InstantiateEvent",
    events::handle_instantiate_event,
);
```

### Step 5: Pick a source.




## How it works

This library defines an `EventHandler` trait, of which implementors allow you to call `handle`

```Rust
pub trait EventHandler<STATE>: DynClone
where
    STATE: Clone,
{
    fn handle(
        &self,
        input: EventHandlerContext<STATE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError>;
}
```

It has a blanket implementation for all functions which match the following constraints:

```Rust
impl<STATE, F> EventHandler<STATE> for F
where
    // Vec<u8> is an array of bytes, representing binary SBOR events.
    F: Fn(EventHandlerContext<STATE>, Vec<u8>) -> Result<(), EventHandlerError>
        + Clone,
    STATE: Clone,
{
    fn handle(
        &self,
        input: EventHandlerContext<STATE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError> {
        // Call self with the input to handle()
        self(input, event)
    }
}
```

We can use this trait to implement event handlers, which can then be registered to a `HandlerRegistry`. The `HandlerRegistry` is passed to a `TransactionStreamProcessor` which

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
