use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{BotCommand, MenuButton};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tokio::sync::RwLock;

use crate::domain::bot_cmd::Command;
use crate::domain::bot_state::BotState;
use crate::domain::page_to_send::PageToSend;
use crate::logic::bot_state::save_bot_state;
use crate::logic::page_sender::*;
use crate::logic::HandlerResult;

pub async fn start(bot: Bot, msg: Message, state: Arc<RwLock<BotState>>) -> HandlerResult {
    let mut state = state.write().await;
    state.subscribers.add(msg.chat.id.0);

    bot.set_my_commands(vec![
        BotCommand::new("first", "first page"),
        BotCommand::new("last", "last available page"),
        BotCommand::new("help", "available commands"),
    ])
    .await?;

    bot.set_chat_menu_button()
        .chat_id(msg.chat.id)
        .menu_button(MenuButton::Commands)
        .await?;
    save_bot_state(&state).await;
    help(bot, msg).await
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn no_page(bot: Bot, id: ChatId, no_str: &str) -> HandlerResult {
    bot.send_message(id, no_str).await?;
    Ok(())
}

pub async fn first(bot: Bot, msg: Message, state: Arc<RwLock<BotState>>) -> HandlerResult {
    match state.read().await.pages.first() {
        None => no_page(bot, msg.chat.id, ":( no first page").await?,
        Some(p) => bot.send_page(PageToSend::old_page(p), msg.chat.id).await?,
    };
    Ok(())
}

pub async fn last(bot: Bot, msg: Message, state: Arc<RwLock<BotState>>) -> HandlerResult {
    match state.read().await.pages.last() {
        None => no_page(bot, msg.chat.id, ":( no last page").await?,
        Some(p) => bot.send_page(PageToSend::old_page(p), msg.chat.id).await?,
    };
    Ok(())
}

async fn by_idx_internal(
    bot: Bot,
    id: ChatId,
    idx: usize,
    state: Arc<RwLock<BotState>>,
) -> HandlerResult {
    match state.read().await.pages.by_idx(idx) {
        None => no_page(bot, id, format!(":( no page at idx {}", idx).as_str()).await?,
        Some(p) => bot.send_page(PageToSend::old_page(p), id).await?,
    };
    Ok(())
}

pub async fn by_idx(
    bot: Bot,
    msg: Message,
    idx: usize,
    state: Arc<RwLock<BotState>>,
) -> HandlerResult {
    by_idx_internal(bot, msg.chat.id, idx, state).await
}

pub async fn nav_callback(
    bot: Bot,
    q: CallbackQuery,
    state: Arc<RwLock<BotState>>,
) -> HandlerResult {
    if let Some(cmd) = &q.data {
        let idx = cmd.as_str().parse::<usize>().unwrap();
        by_idx_internal(bot, q.message.unwrap().chat.id, idx, state).await?;
    }

    Ok(())
}
