use radix_engine_toolkit::functions::scrypto_sbor::{
    encode_string_representation, StringRepresentation,
};
use radix_event_stream::decoder::HandlerRegistry;
use radix_event_stream::models::PoolType;
use radix_event_stream::poolstore::PoolStore;

use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;

use log::{info, warn};
use radix_client::gateway::models::*;
use radix_client::GatewayClientBlocking;
use radix_event_stream::basicv0;
use radix_event_stream::decoder::DecoderRegistry;
use radix_event_stream::models::*;

fn main() {
    let settings = radix_event_stream::settings::Settings::init();
    // Settings::init() brings .env into the environment variables of the process.
    // That's why LOG_LEVEL works here. Maybe this is not very transparent.
    env_logger::init_from_env(env_logger::Env::new().filter("LOG_LEVEL"));

    info!("Starting fetcher");

    let pool_store = PoolStore::new(settings.network);
    let pool_store_rc = Rc::new(RefCell::new(pool_store));

    pool_store_rc.borrow_mut().add_package_address(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        PoolType::BasicV0,
    );

    let mut decoder_registry = HandlerRegistry::new();

    // Register decoder for each event type
    decoder_registry.add_handler(Box::new(
        basicv0::events::InstantiateEventDecoder {
            pool_store: Rc::clone(&pool_store_rc),
        },
    ));
    // decoder_registry.add_handler(Box::new(basicv0::events::SwapEventDecoder {
    //     pool_store: Rc::clone(&pool_store_rc),
    // }));
    // decoder_registry.add_handler(Box::new(
    //     basicv0::events::ContributionEventDecoder {
    //         pool_store: Rc::clone(&pool_store_rc),
    //     },
    // ));
    // decoder_registry.add_handler(Box::new(
    //     basicv0::events::RedemptionEventDecoder {
    //         pool_store: Rc::clone(&pool_store_rc),
    //     },
    // ));

    let client = GatewayClientBlocking::new(settings.radix_gateway_url);

    // Create a new transaction stream
    let mut stream =
        client.new_transaction_stream(TransactionStreamRequestBody {
            from_ledger_state: Some(LedgerStateSelector {
                state_version: Some(settings.start_from_state_version),
                ..Default::default()
            }),
            limit_per_page: Some(settings.limit_per_page),
            affected_global_entities_filter: None,
            opt_ins: Some(TransactionStreamOptIns {
                receipt_events: true,
                ..Default::default()
            }),
            order: Some(Order::Asc),
            kind_filter: TransactionKindFilter::User,
            ..Default::default()
        });
    loop {
        let resp = match stream.next() {
            Ok(resp) => {
                if resp.items.is_empty() {
                    info!("No more transactions, sleeping for 1 second...");
                    sleep(std::time::Duration::from_secs(1));
                    continue;
                }
                resp
            }
            // This case could happen when something goes wrong with the request,
            // but in practice it happens due to a very specific cursoring issue.
            // Should fix that in radix-clients later but for now we just retry.
            Err(err) => {
                warn!(
                    "Error while getting transactions, trying again: {:?}",
                    err
                );
                sleep(std::time::Duration::from_secs(1));
                continue;
            }
        };

        resp.items.iter().for_each(|item| {
            info!("State version: {}", item.state_version);

            let events = item.receipt.clone().unwrap().events.unwrap();

            events.iter().for_each(|event| {
                let start_time = std::time::Instant::now();
                let decoded = match decoder_registry.decode(&event) {
                    Some(decoded) => {
                        info!("Decoded: {:?}", decoded);
                        println!("Decoding took: {:#?}", start_time.elapsed());
                        decoded
                    }
                    None => {
                        return;
                    }
                };

                decoded.update_pool_store(&mut pool_store_rc.borrow_mut());
            })
        });
    }
}

#[cfg(test)]
mod tests {
    use fetcher::basicv0;

    use super::*;
    use fetcher::EventName;

    #[test]
    fn test_event_name_derive() {
        println!("{}", basicv0::events::InstantiateEvent::event_name()); // Prints "MyEvent"
    }
}
