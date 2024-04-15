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

### ✅ Async:

Supports asynchronous operations inside handlers for efficient queries.

## Background

[Radix](https://www.radixdlt.com) is a platform for decentralized applications, specifically built for DeFi. Each smart contract, called a component on Radix, can emit events when transactions happen, including custom events defined by the author of the smart contract. An event may look somewhat like this:

```Rust
#[derive(ScryptoSbor, ScryptoEvent)]
struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}
```

These events are recorded inside of transactions in-sequence and stored on the Radix ledger. This allows developers to track the state of an application by reading events as they happen, and processing them in some specified way. That's what this library aims to achieve.

Events on Radix are encoded as SBOR (Scrypto Binary-friendly Object Representation), a custom format supporting binary and json representations. Radix provides Rust crates which can decode SBOR directly into the events as we defined them in a blueprint. This is possible because Scrypto, the programming language of Radix smart contracts, is based on Rust.

Each event has an **emitter**. This is the on-ledger entity that emitted the event. Events also have a **name**, which is the name of the event and its data type on ledger. These two identifiers are enough to map incoming events to handlers.

## General usage.

### Step 1: copy/paste event definitions.

```Rust
#[derive(ScryptoSbor, Debug)]
struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    context_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}
```

Above, we see an event definition used in one of Ociswap's Basic pools. It derives at the very least `radix_engine_common::ScryptoSbor`, which is needed to decode it from binary Scrypto SBOR. Copy this over to your project.

### Step 2: Define a global state.

This will be mutably shared with every transaction handler. It should at least implement Clone. If you need to share this data with other pieces of code you may choose to store items wrapped in Rc, RefCell, Arc, Mutex, etc.

```rust
#[derive(Clone)]
struct State {
    instantiate_events_seen: u64
}
```

### Step 3: implement event handlers.

Write one or multiple event handlers which conform to the predefined handler signature:

```Rust
// A macro from the crate which transforms the handler function
// into a representation that is usable for the framework.
#[event_handler]
// Name of your handler
async fn event_handler_name(
    // Context the handler will get from the framework.
    // This includes the current ledger transaction we're in,
    // the raw event, the global state, and the transaction context.
    context: EventHandlerContext<YOUR_STATE>,
    // The decoded event struct as defined in your smart contract.
    event: EVENT_STRUCT,
) -> Result<(), EventHandlerError> {
    // Handle the event here.

    // Possible errors to return:
    // Retry handling the current event
    return Err(EventHandlerError::EventRetryError(
        anyhow!("Retry event because of...")
    ));
    // Retry handling the current transaction
    return Err(EventHandlerError::TransactionRetryError(
        anyhow!("Retry transaction because of...")
    ));
    // Stop the stream
    return Err(EventHandlerError::UnrecoverableError(
        anyhow!("Stream failed because of...")
    ));
    // Everything's ok!
    Ok(())
}
```

It must take in the `EventHandlerContext`, which stores things like the current ledger transaction and the global state defined in step 2. I also takes the decoded event type as we copied over from our blueprint in step 1.

The `EventHandler` trait actually defines handlers to take in a `Vec<u8>` instead of the event type itself, but the `#[event_handler]` macro expands the function to handle decoding of the event for you.

A concrete example:

```Rust
#[event_handler]
async fn handle_instantiate_event(
    context: EventHandlerContext<AppState>,
    event: InstantiateEvent,
) -> Result<(), EventHandlerError> {
    info!(
        "Handling the {}th instantiate event: {:#?}",
        context.state.instantiate_events_seen, event
    );
    context.state.instantiate_events_seen += 1;
    Ok(())
}
```

This handler counts the amount of `InstantiateEvents` seen inside an app state variable as we go through the ledger.

As shown above, it is possible to return errors from the event:

```rust
enum EventHandlerError {
    /// The event handler encountered an error and
    /// should be retried directly.
    /// This shouldn't be propagated up to the transaction handler.
    EventRetryError(anyhow::Error),
    /// The event handler encountered an error and
    /// the whole transaction should be retried.
    TransactionRetryError(anyhow::Error),
    /// The event handler encountered an unrecoverable
    /// error and the process should exit.
    UnrecoverableError(anyhow::Error),
}
```

By returning different errors, you may control how the stream behaves. It can retry handling the current event directly, retry the whole transaction handler, or exit completely. **Beware:** This could mean your handlers will be called multiple times if an error occurs. When using this option, ensure your handlers are somehow idempotent or atomic, so that running them multiple times is fine.

### Step 4: Register handlers.

Create a handler registry:

```Rust
let mut handler_registry: HandlerRegistry<AppState> =
    HandlerRegistry::new();
```

Add any handlers to the registry, identified by emitters and event names. In this case, we would like to handle `InstantiateEvent` events emitted by Ociswap's Basic pool package address.

```rust
// Add the instantiate event handler to the registry
handler_registry.add_handler(
    "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
    "InstantiateEvent",
    handle_instantiate_event,
);
```

Note that you can also register new handlers inside a handler. This is necessary when a new component is instantiated, to register handlers for that component.

### Step 5: Pick a source.

The library holds a few different transaction stream sources out of the box: A Radix Gateway stream based on our radix-client crate, a database stream which fetches directly from the Gateway PostgreSQL database, a file stream, and a channel stream. It is also possible to implement custom streams.

Let's use the gateway stream:

```Rust
// For some of the included streams, a builder pattern is used.
let stream = GatewayTransactionStream::new()
    .gateway_url("https://mainnet.radixdlt.com".to_string())
    .from_state_version(1919391)
    .buffer_capacity(1000)
    .limit_per_page(100);
```

A transaction stream implements the `TransactionStream` trait. This trait has a `start()` and a `stop()` method. `start()` may start a new asynchronous task, which pushes `Transaction` items to the `Receiver` which is returned by the method. Having a channel allows the transaction stream to fetch and buffer transactions independently of the rest of the framework.

```rust
trait TransactionStream {
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error>;
    async fn stop(&mut self);
}
```

See `stream.rs` for more information.

### Step 6: Define a transaction handler and transaction context. (Optional)

To make the transaction stream have any kind of sense of ledger transactions, we must implement a custom transaction handler. This will allow us to do transaction-level operations. For example, if we want to store events in a database, and we want to push events to our database per ledger transaction atomically, we might want to use database transactions. Each time we get a transaction from the stream, we should start a database transaction and try to commit it after all the events have been handled. We can use a custom transaction handler for this.

Custom transaction handlers can pass a custom transaction context to event handlers. This gives event handlers access to a database transaction for example, so that each handler can do inserts inside that transaction. Let's define a custom transaction context struct:

```rust
// pseudocode
struct TransactionContext {
    tx: YOUR_DATABASE_TRANSACTION
}
```

It should again conform to the predefined signature:

```Rust
// A macro from the crate which transforms the handler function
// into a representation that is usable for the framework.
#[transaction_handler]
// Name of your handler
async fn transaction_handler_name(
    // Context the handler will get from the framework.
    // This includes the current ledger transaction we're in
    // and the global state. It is parametrized by the
    // app state and the transaction context type, but the context is optional,
    // and defaults to the unit type.
    context: TransactionHandlerContext<YOUR_STATE, YOUR_TRANSACTION_CONTEXT_TYPE>,
) -> Result<(), TransactionHandlerError> {
    // Do something like start a database transaction
    let mut transaction_context = TransactionContext { tx: start_transaction() }

    // Handle the events inside the incoming transaction.
    // We provide a simple method for this.
    context
        .transaction
        .process_events(
            context.state,
            context.handler_registry,
            // the transaction context is passed in
            &mut transaction_context,
        )
        // EventHandlerErrors can be cast into TransactionHandlerErrors,
        // and the framework will handle these appropriately.
        // So, best to propagate these with the ? operator.
        .await?;

    // Possible errors to return:
    // Retry handling the current transaction
    return Err(EventHandlerError::TransactionRetryError(
        anyhow!("Retry transaction because of...")
    ));
    // Stop the stream
    return Err(EventHandlerError::UnrecoverableError(
        anyhow!("Stream failed because of...")
    ));
    // Everything's ok!
    Ok(())
}
```

The `TransactionHandlerContext` holds a reference to the incoming transaction. A method called `process_events` is implemented on this struct. Calling it will iterate through the events inside the transactions and process the events which have handlers registered. It is highly recommended to use this method in your transaction handler. It is possible to implement your own loop, but it is an integral part of the library and also handles the event retry logic and some logging.

Simplest concrete example:

```rust
#[transaction_handler]
async fn transaction_handler(
    context: TransactionHandlerContext<AppState, ()>,
) -> Result<(), TransactionHandlerError> {
    // Do something before handling events
    context
        .transaction
        .process_events(context.state, context.handler_registry)?;
    // Do something after handling events
    Ok(())
}
```

### Step 7: Run the stream processor.

The `TransactionStreamProcessor` is what ties everything together. It is responsible for getting new transactions from the stream we selected, and calling the transaction handler which in turn calls the event handlers.

Use the `run_with` method to start the processor, passing in the components we created and the initial app state.

```rust
TransactionStreamProcessor::run_with(
            stream,
            handler_registry,
            transaction_handler,
            State {
                instantiate_events_seen: 0
            },
        );
```

There also exists a `SimpleTransactionStreamProcessor`, which does not require a transaction handler. It simply calls the `process_events` method from the previous step and nothing else. It's easier to set up and recommended in the case where you do not need transaction-level handling.

# More info

For more examples, please see the `/examples` directory, or ask us on telegram/discord if you have any specific questions! We'd be happy to help.
