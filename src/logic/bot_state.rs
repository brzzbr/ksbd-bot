use crate::domain::bot_state::BotState;
use crate::logic::pages_state::{load_pages_state, restore_pages_from, save_pages_state};
use crate::logic::subs_state::{load_subs_state, save_subs_state};
use futures::StreamExt;

pub async fn load_bot_state() -> BotState {
    let pages_state = load_pages_state().await;
    if let Some((idx, url)) = pages_state.start_from() {
        log::info!("restoring full state, from {}", idx);
        restore_pages_from(idx, url)
            .fold(pages_state.clone(), move |mut state, page| {
                async move {
                    state.add_page(page);
                    save_pages_state(&state).await;
                    state
                }
            })
            .await;
        log::info!("state has been restored");
    }
    let pages_state = load_pages_state().await;
    let subs_state = load_subs_state().await;

    BotState {
        pages: pages_state,
        subscribers: subs_state,
    }
}

pub async fn save_bot_state(state: &BotState) {
    save_pages_state(&state.pages).await;
    save_subs_state(&state.subscribers).await;
}
