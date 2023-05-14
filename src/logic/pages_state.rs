use std::path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use futures::{stream, Stream, StreamExt};
use lazy_static::lazy_static;
use teloxide::Bot;
use tokio::fs;
use tokio::sync::RwLock;

use crate::cfg::DATA_PATH;
use crate::domain::bot_state::BotState;
use crate::domain::ksbd_page::KsbdPage;
use crate::domain::pages_state::PagesState;
use crate::logic::bot_flow::send_page;
use crate::logic::ksbd_page::{download_imgs, request_page};

lazy_static! {
    static ref STATE_PATH: String = format!("{}/pages_state.txt", DATA_PATH.as_str());
}

// if it fails, it fails!
pub async fn load_pages_state() -> PagesState {
    match path::Path::new(STATE_PATH.as_str()).exists() {
        false => PagesState::default(),
        true => fs::read_to_string(STATE_PATH.as_str())
            .await
            .map_err(|e| e.to_string())
            .and_then(|s| PagesState::from_str(s.as_str()))
            .unwrap(),
    }
}

// if it fails, it fails!
pub async fn save_pages_state(state: &PagesState) {
    fs::write(STATE_PATH.as_str(), state.to_string())
        .await
        .unwrap()
}

pub fn restore_pages_from(idx: usize, url: String) -> impl Stream<Item = KsbdPage> {
    async fn get_page(
        idx: usize,
        maybe_url: Option<String>,
    ) -> Option<(KsbdPage, (usize, Option<String>))> {
        if let Some(url) = maybe_url {
            let start = Instant::now();
            let page = request_page(idx, &url).await.unwrap();
            let next = page.next.clone();
            download_imgs(&page).await.unwrap();
            let duration = start.elapsed();

            log::info!(
                "page got: [idx: {}, url: {}, elapsed: {}ms]",
                idx,
                url,
                duration.as_millis()
            );

            Some((page, (idx + 1, next)))
        } else {
            None
        }
    }

    stream::unfold((idx, Some(url)), |(idx, maybe_url)| {
        get_page(idx, maybe_url)
    })
}

pub async fn check_new_page_and_send(state: &Arc<RwLock<BotState>>, bot: &Bot) {
    let maybe_last;
    let subs_chat_ids;
    {
        let state_for_read = state.read().await;
        maybe_last = state_for_read.pages.last().cloned();
        subs_chat_ids = state_for_read.subscribers.clone().chat_ids();
    }

    match maybe_last {
        Some(p) if p.next.is_none() => {
            log::info!("requesting...");
            match request_page(p.idx, p.url.as_str()).await {
                Err(e) => log::error!("{}", e),
                Ok(p) => {
                    if let Some(new_url) = p.next {
                        let new_pages = restore_pages_from(p.idx + 1, new_url)
                            .collect::<Vec<_>>()
                            .await;

                        for chat_id in subs_chat_ids {
                            for p in new_pages.clone() {
                                let _ = send_page(true, bot.clone(), chat_id, &p).await;
                            }
                        }

                        let mut state_to_write = state.write().await;
                        state_to_write.pages.add_pages(new_pages.clone());
                        save_pages_state(&state_to_write.pages).await
                    }
                }
            }
        }
        Some(_) => log::error!("last page has next, should not"),
        None => log::warn!("no last page to watch from"),
    };
}
