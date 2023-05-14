use std::sync::Arc;
use std::time;

use dotenv::dotenv;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateHandler};
use teloxide::prelude::*;
use tokio::fs;
use tokio::sync::RwLock;

use crate::cfg::DATA_PATH;
use crate::domain::bot_cmd::Command;
use crate::logic::bot_flow::*;
use crate::logic::bot_state::load_bot_state;
use crate::logic::pages_state::check_new_page_and_send;

mod cfg;
mod domain;
mod logic;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("reading cfg, loading state, doing initialization mumbo-jumbo...");
    fs::create_dir_all(DATA_PATH.as_str()).await.unwrap();
    let bot_state = load_bot_state().await;
    let shared_bot_state = Arc::new(RwLock::new(bot_state));
    let bot = Bot::from_env();

    log::info!("starting new pages watcher...");

    let state_for_updater = shared_bot_state.clone();
    let bot_for_updater = bot.clone();
    tokio::spawn(async move {
        let delay = time::Duration::from_secs(300);
        log::info!("gonna request for a new page(s)...");
        loop {
            check_new_page_and_send(&state_for_updater, &bot_for_updater).await;
            tokio::time::sleep(delay).await
        }
    });

    log::info!("ksbd bot started...");

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![shared_bot_state, InMemStorage::<()>::new()])
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
