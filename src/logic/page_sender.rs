use crate::domain::page_to_send::PageToSend;
use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile};
use teloxide::Bot;
use crate::logic::HandlerResult;

#[async_trait]
pub trait PageSender {
    async fn send_page(&self, p: PageToSend, to: ChatId) -> HandlerResult;
}

#[async_trait]
impl PageSender for Bot {
    async fn send_page(&self, p: PageToSend, to: ChatId) -> HandlerResult {
        if p.is_new {
            self.send_message(to, "🎉🎉🎉 GREAT NEWS!! NEW PAGE IS ON THE WAY 🎉🎉🎉")
                .await?;
        }

        let mut but_row = Vec::<InlineKeyboardButton>::new();
        if p.idx > 0 {
            but_row.push(InlineKeyboardButton::callback(
                "PREV",
                (p.idx - 1).to_string(),
            ));
        }
        if p.has_next {
            but_row.push(InlineKeyboardButton::callback(
                "NEXT",
                (p.idx + 1).to_string(),
            ));
        }

        let buttons = vec![but_row];
        let markup = InlineKeyboardMarkup::new(buttons);

        let txt_to_send = p.title.replace("%09", "\t").replace("%0D%0A", "\n");
        if !txt_to_send.trim().is_empty() {
            self.send_message(to, txt_to_send).await?;
        }

        let txt_to_send = p.text.replace("%09", "\t").replace("%0D%0A", "\n");
        if !txt_to_send.trim().is_empty() {
            for img_file in p.img_files() {
                self.send_document(to, InputFile::file(img_file)).await?;
            }
            self.send_message(to, txt_to_send)
                .reply_markup(markup)
                .await?;
        } else {
            // it has to have at least one img, hence unwrap
            let img_files = p.img_files();
            let (last, first) = img_files.as_slice().split_last().unwrap();

            for img_file in first {
                self.send_document(to, InputFile::file(img_file)).await?;
            }

            self.send_document(to, InputFile::file(last))
                .reply_markup(markup)
                .await?;
        }

        Ok(())
    }
}
