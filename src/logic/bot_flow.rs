use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, MenuButton};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

use crate::domain::bot_cmd::Command;
use crate::domain::page_to_send::PageToSend;
use crate::logic::bot_state::BotStateManager;
use crate::logic::page_sender::*;
use crate::logic::HandlerResult;

pub async fn start(
    state: Arc<dyn BotStateManager + Send + Sync>,
    bot: Bot,
    msg: Message,
) -> HandlerResult {
    state.add_subs(msg.chat.id).await;

    bot.set_my_commands(vec![
        BotCommand::new("first", "first page"),
        BotCommand::new("last", "last available page"),
        BotCommand::new("jump", "jump to some page"),
        BotCommand::new("help", "available commands"),
    ])
    .await?;

    log::info!(
        "new user subscribed: [{}, {:?}]",
        msg.chat.id,
        msg.chat.username()
    );

    bot.set_chat_menu_button()
        .chat_id(msg.chat.id)
        .menu_button(MenuButton::Commands)
        .await?;

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

pub async fn first(
    state: Arc<dyn BotStateManager + Send + Sync>,
    bot: Bot,
    msg: Message,
) -> HandlerResult {
    match state.first().await {
        None => no_page(bot, msg.chat.id, ":( no first page").await?,
        Some(p) => {
            bot.send_full_page(PageToSend::old_page(p), msg.chat.id)
                .await?
        }
    };
    Ok(())
}

pub async fn last(
    state: Arc<dyn BotStateManager + Send + Sync>,
    bot: Bot,
    msg: Message,
) -> HandlerResult {
    match state.last().await {
        None => no_page(bot, msg.chat.id, ":( no last page").await?,
        Some(p) => {
            bot.send_full_page(PageToSend::old_page(p), msg.chat.id)
                .await?
        }
    };
    Ok(())
}

async fn by_idx_internal(
    state: Arc<dyn BotStateManager + Send + Sync>,
    bot: Bot,
    id: ChatId,
    idx: usize,
) -> HandlerResult {
    match state.by_idx(idx).await {
        None => no_page(bot, id, format!(":( no page at idx {}", idx).as_str()).await?,
        Some(p) => bot.send_full_page(PageToSend::old_page(p), id).await?,
    };
    Ok(())
}

pub async fn nav_callback(
    state: Arc<dyn BotStateManager + Send + Sync>,
    bot: Bot,
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(cmd) = &q.data {
        let maybe_cmd_and_idx = cmd
            .split_once('-')
            .and_then(|(cmd, idx_str)| idx_str.parse::<usize>().ok().map(|idx| (cmd, idx)));

        let chat_id = q.message.unwrap().chat.id;

        match maybe_cmd_and_idx {
            Some(("n", idx)) => by_idx_internal(state, bot, chat_id, idx).await?,
            _ => log::warn!("unexpected callback {}", cmd),
        }
    }

    Ok(())
}

pub async fn jump_menu(
    state: Arc<dyn BotStateManager + Send + Sync>,
    bot: Bot,
    msg: Message,
) -> HandlerResult {
    static PAGES_IN_ROW: isize = 100;
    static BTNS_IN_ROW: isize = 4;
    static PAGES_IN_BTN: isize = PAGES_IN_ROW / BTNS_IN_ROW;

    let last_idx = state.last_idx().await.unwrap_or(0) as isize;

    let btn_on_idx = |idx| InlineKeyboardButton::callback(format!("{}", idx), format!("n-{}", idx));

    let btn_rows = (0..=last_idx / PAGES_IN_ROW + 1).fold(
        vec![vec![InlineKeyboardButton::callback("FIRST", "n-0")]],
        |mut acc, row| {
            let processed = row * PAGES_IN_ROW;
            let rest = last_idx - processed;

            let btn_row = (1..=BTNS_IN_ROW).fold(vec![], |mut inner_acc, rr| {
                if rest - rr * PAGES_IN_BTN > 0 {
                    inner_acc.push(btn_on_idx(processed + rr * PAGES_IN_BTN));
                }
                inner_acc
            });

            match btn_row.is_empty() {
                true => acc.push(vec![InlineKeyboardButton::callback(
                    "LAST",
                    format!("n-{}", last_idx),
                )]),
                false => acc.push(btn_row),
            }

            acc
        },
    );

    bot.send_message(msg.chat.id, "JUMP TO")
        .reply_markup(InlineKeyboardMarkup::new(btn_rows))
        .await?;

    Ok(())
}
