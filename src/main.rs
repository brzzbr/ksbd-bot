use std::sync::Arc;
use std::time;

use dotenv::dotenv;
use futures::StreamExt;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateHandler};
use teloxide::prelude::*;
use tokio::fs;

use crate::cfg::DATA_PATH;
use crate::domain::bot_cmd::Command;
use crate::domain::page_to_send::PageToSend;
use crate::logic::bot_flow::*;
use crate::logic::bot_state::BotStateManager;
use crate::logic::bot_state::{BotStateManagerImpl, BotStateManagerInit};
use crate::logic::page_sender::PageSender;
use crate::logic::scraper::KsbdScraper;
use crate::logic::scraper::KsbdScraperImpl;

mod cfg;
mod domain;
mod logic;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("reading cfg, loading state, doing initialization mumbo-jumbo...");
    fs::create_dir_all(DATA_PATH.as_str()).await.unwrap();

    let scraper = KsbdScraperImpl {};

    let bot_state_manager = BotStateManagerImpl::init(&scraper).await;
    let bot = Bot::from_env();

    log::info!("starting new pages watcher...");
    let bot_for_updater = bot.clone();
    let bot_state_manager_for_updater = bot_state_manager.clone();
    tokio::spawn(async move {
        let delay = time::Duration::from_secs(300);
        log::info!("gonna request for a new page(s)...");
        loop {
            check_new_page_and_send(&bot_state_manager_for_updater, &bot_for_updater, &scraper)
                .await;
            tokio::time::sleep(delay).await
        }
    });

    log::info!("ksbd bot started...");

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            // need to cast, otherwise dptree is unable to find manager dependency
            Arc::new(bot_state_manager) as Arc<dyn BotStateManager + Send + Sync>,
            InMemStorage::<()>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("ksbd bot stopped...");
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::First].endpoint(first))
        .branch(case![Command::Last].endpoint(last))
        .branch(case![Command::ByIdx { idx }].endpoint(by_idx));

    let message_handler = Update::filter_message().branch(command_handler);

    let callback_query_handler = Update::filter_callback_query().endpoint(nav_callback);

    dialogue::enter::<Update, InMemStorage<()>, (), _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

// screw it. I'm done. gonna leave it like this. just a function in a main. hardcore to the mega.
async fn check_new_page_and_send(
    state: &impl BotStateManager,
    sender: &impl PageSender,
    scraper: &impl KsbdScraper,
) {
    match state.maybe_last().await {
        Some(p) if p.next.is_none() => {
            log::info!("requesting...");
            match scraper.request_page(p.idx, p.url.as_str()).await {
                Err(e) => log::error!("{}", e),
                Ok(p) => {
                    if let Some(new_url) = p.next {
                        let new_pages = scraper
                            .pages_from(p.idx + 1, new_url)
                            .collect::<Vec<_>>()
                            .await;

                        for chat_id in state.subs_chat_ids().await {
                            for p in new_pages.clone() {
                                let _ = sender.send_page(PageToSend::fresh_page(p), chat_id).await;
                            }
                        }

                        state.add_pages(new_pages).await;
                    }
                }
            }
        }
        Some(_) => log::error!("last page has next, should not"),
        None => log::warn!("no last page to watch from"),
    };
}
