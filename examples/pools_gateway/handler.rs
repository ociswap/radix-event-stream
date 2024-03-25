use crate::basicv0::events::{
    handle_contribution_event, handle_instantiate_event,
    handle_redemption_event, handle_swap_event, ContributionEvent,
    InstantiateEvent, RedemptionEvent, SwapEvent,
};
use radix_event_stream::map_handlers;

#[derive(Debug, Clone)]
pub struct AppState {
    pub number: u64,
}

map_handlers!(
    Handler {
        BasicV0 {
            InstantiateEvent => handle_instantiate_event,
            SwapEvent => handle_swap_event,
            ContributionEvent => handle_contribution_event,
            RedemptionEvent => handle_redemption_event,
        }
    },
    AppState
);
