use async_trait::async_trait;
use reqwest::Url;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile};
use teloxide::Bot;

use crate::domain::page_to_send::PageToSend;
use crate::logic::HandlerResult;

#[async_trait]
pub trait PageSender {
    async fn send_full_page(&self, p: PageToSend, to: ChatId) -> HandlerResult;
}

#[async_trait]
impl PageSender for Bot {
    async fn send_full_page(&self, p: PageToSend, to: ChatId) -> HandlerResult {
        if p.is_new {
            self.send_message(to, "🎉🎉🎉 GREAT NEWS!! NEW PAGE IS ON THE WAY 🎉🎉🎉")
                .await?;
        }

        let imgs = p.img_files().clone();
        if let Some(title) = &p.title {
            self.send_message(to, title).await?;
        }

        match &p.text.is_empty() {
            true => {
                // it has to have at least one img, hence unwrap
                let (last, first) = imgs.as_slice().split_last().unwrap();

                for img_file in first {
                    self.send_document(to, InputFile::file(img_file)).await?;
                }

                let markup = InlineKeyboardMarkup::new(vec![nav_btns(&p)]);
                self.send_document(to, InputFile::file(last))
                    .reply_markup(markup)
                    .await?;
            }
            false => {
                for img_file in imgs {
                    self.send_document(to, InputFile::file(img_file)).await?;
                }

                let re_arranged_txts = resize_text(&p.text, 2000);
                let (last_txt, first_txts) = re_arranged_txts.split_last().unwrap();

                for txt in first_txts {
                    self.send_message(to, txt)
                        .reply_markup(InlineKeyboardMarkup::new(vec![translate_btn(txt)]))
                        .await?;
                }

                self.send_message(to, last_txt)
                    .reply_markup(InlineKeyboardMarkup::new(vec![
                        translate_btn(last_txt),
                        nav_btns(&p),
                    ]))
                    .await?;
            }
        }

        Ok(())
    }
}

fn resize_text(txt: &[String], max_len: usize) -> Vec<String> {
    txt.iter()
        // could fail if txt element itself is bigger than max_len
        // will fix later. or won't.
        .fold(vec![], |mut acc, t| match (acc.pop(), t.len()) {
            (None, _) => {
                acc.push(t.to_string());
                acc
            }
            (Some(last), l) if (l + last.len()) < max_len => {
                acc.push(format!("{}\n\n{}", last, t));
                acc
            }
            (Some(last), _) => {
                acc.push(last);
                acc.push(t.to_string());
                acc
            }
        })
}

fn translate_btn(txt: &str) -> Vec<InlineKeyboardButton> {
    let txt_url_encoded = urlencoding::encode(txt);

    vec![InlineKeyboardButton::url(
        "TRANSLATE",
        Url::parse(
            format!(
                "https://translate.google.com/?sl=en&tl=ru&text={}&op=translate",
                txt_url_encoded
            )
            .as_str(),
        )
        .unwrap(),
    )]
}

fn nav_btns(p: &PageToSend) -> Vec<InlineKeyboardButton> {
    let mut nav_but_row = Vec::<InlineKeyboardButton>::new();
    if p.idx > 0 {
        nav_but_row.push(InlineKeyboardButton::callback(
            "PREV",
            format!("n-{}", p.idx - 1),
        ));
    }
    if p.has_next {
        nav_but_row.push(InlineKeyboardButton::callback(
            "NEXT",
            format!("n-{}", p.idx + 1),
        ));
    }
    nav_but_row
}
