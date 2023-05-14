use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{
    BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MenuButton,
};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tokio::sync::RwLock;

use crate::domain::bot_cmd::Command;
use crate::domain::bot_state::BotState;
use crate::domain::ksbd_page::KsbdPage;
use crate::logic::bot_state::save_bot_state;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_page(is_new: bool, b: Bot, id: ChatId, p: &KsbdPage) -> HandlerResult {
    let p = p.clone();

    if is_new {
        b.send_message(id, "🎉🎉🎉 GREAT NEWS!! NEW PAGE IS ON THE WAY 🎉🎉🎉")
            .await?;
    }

    let mut but_row = Vec::<InlineKeyboardButton>::new();
    if p.idx > 0 {
        but_row.push(InlineKeyboardButton::callback(
            "PREV",
            (p.idx - 1).to_string(),
        ));
    }
    if p.next.is_some() {
        but_row.push(InlineKeyboardButton::callback(
            "NEXT",
            (p.idx + 1).to_string(),
        ));
    }

    let buttons = vec![but_row];
    let markup = InlineKeyboardMarkup::new(buttons);

    let txt_to_send = p.title.replace("%09", "\t").replace("%0D%0A", "\n");
    if !txt_to_send.trim().is_empty() {
        b.send_message(id, txt_to_send).await?;
    }

    let txt_to_send = p.text.replace("%09", "\t").replace("%0D%0A", "\n");
    if !txt_to_send.trim().is_empty() {
        for img_file in p.img_files() {
            b.send_document(id, InputFile::file(img_file)).await?;
        }
        b.send_message(id, txt_to_send).reply_markup(markup).await?;
    } else {
        // it has to have at least one img, hence unwrap
        let img_files = p.img_files();
        let (last, first) = img_files.as_slice().split_last().unwrap();

        for img_file in first {
            b.send_document(id, InputFile::file(img_file)).await?;
        }

        b.send_document(id, InputFile::file(last))
            .reply_markup(markup)
            .await?;
    }

    Ok(())
}

pub async fn start(bot: Bot, msg: Message, state: Arc<RwLock<BotState>>) -> HandlerResult {
    let mut state = state.write().await;
    state.subscribers.add(msg.chat.id.0);

    bot.set_my_commands(vec![
        BotCommand::new("first", "first page"),
        BotCommand::new("last", "last available page"),
        BotCommand::new("help", "shows available commands"),
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
        Some(p) => send_page(false, bot, msg.chat.id, p).await?,
    };
    Ok(())
}

pub async fn last(bot: Bot, msg: Message, state: Arc<RwLock<BotState>>) -> HandlerResult {
    match state.read().await.pages.last() {
        None => no_page(bot, msg.chat.id, ":( no last page").await?,
        Some(p) => send_page(false, bot, msg.chat.id, p).await?,
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
        Some(p) => send_page(false, bot, id, p).await?,
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
