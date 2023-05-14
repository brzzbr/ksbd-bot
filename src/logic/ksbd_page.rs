use futures::future::join_all;
use std::fmt;

use crate::cfg::DATA_PATH;
use futures::TryFutureExt;
use lazy_static::lazy_static;
use scraper::{Html, Selector};

use crate::domain::ksbd_page::KsbdPage;

#[derive(Debug)]
pub enum GetPageError {
    RequestErr(String, reqwest::Error),
    PageImgErr(String, image::ImageError),
    NoImg(String),
}

impl GetPageError {
    pub fn no_img(url: &str) -> GetPageError {
        GetPageError::NoImg(url.to_string())
    }

    pub fn req_err(url: &str, err: reqwest::Error) -> GetPageError {
        GetPageError::RequestErr(url.to_string(), err)
    }

    pub fn img_err(url: &str, err: image::ImageError) -> GetPageError {
        GetPageError::PageImgErr(url.to_string(), err)
    }
}

impl fmt::Display for GetPageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetPageError::RequestErr(url, err) => {
                write!(f, "Failed to get url ({}): {}", url, err)
            }
            GetPageError::PageImgErr(url, err) => {
                write!(f, "Failed to get img ({}): {}", url, err)
            }
            GetPageError::NoImg(url) => {
                write!(f, "No img ({})", url)
            }
        }
    }
}

lazy_static! {
    static ref SELECTOR_IMG: Selector = Selector::parse("#comic img").unwrap();
    static ref SELECTOR_NEXT: Selector = Selector::parse("#sidebar-over-comic > div > table > tbody > tr > td.comic_navi_right > a.navi.comic-nav-next.navi-next").unwrap();
    static ref SELECTOR_ENTRY: Selector = Selector::parse(".entry p").unwrap();
}

pub async fn request_page(idx: usize, url: &str) -> Result<KsbdPage, GetPageError> {
    let document: Html = reqwest::get(url)
        .and_then(|r| r.text())
        .map_err(|e| GetPageError::req_err(url, e))
        .map_ok(|t| Html::parse_document(&t))
        .await?;

    let (maybe_title, maybe_img_urls) =
        document
            .select(&SELECTOR_IMG)
            .fold((None, vec![]), |(title, mut imgs), img_el| {
                let maybe_new_title = img_el.value().attr("title");
                let maybe_img_url = img_el.value().attr("src").ok_or(GetPageError::no_img(url));
                imgs.push(maybe_img_url);

                (title.or(maybe_new_title), imgs)
            });

    let title = maybe_title
        .unwrap_or("NO TITLE")
        .replace("\t", "%09")
        .replace("\n", "%0D%0A");

    let img_urls = maybe_img_urls
        .into_iter()
        .flatten()
        .map(|u| u.to_string())
        .collect::<Vec<_>>();

    let text = document
        .select(&SELECTOR_ENTRY)
        .map(|e| {
            e.text()
                .next()
                .unwrap_or("")
                .replace("\t", "%09")
                .replace("\n", "%0D%0A")
        })
        .collect::<Vec<_>>()
        .join("%0D%0A%0D%0A");

    let next_url = document
        .select(&SELECTOR_NEXT)
        .next()
        .and_then(|e| e.value().attr("href"));

    Ok(KsbdPage {
        idx,
        title: title.to_string(),
        url: url.to_string(),
        imgs: img_urls,
        next: next_url.map(|u| u.to_string()),
        text,
    })
}

pub async fn download_imgs(page: &KsbdPage) -> Result<(), GetPageError> {
    let futs = page
        .imgs
        .iter()
        .enumerate()
        .map(|(idx, img)| async move {
            let url = img.as_str();
            let img_bytes = reqwest::get(url)
                .and_then(|r| r.bytes())
                .map_err(|e| GetPageError::req_err(url, e))
                .await?;

            image::load_from_memory(&img_bytes)
                .and_then(|img| {
                    let img_path = format!("{}/{}-{}.png", DATA_PATH.as_str(), page.idx, idx);
                    img.save(img_path)
                })
                .map_err(|e| GetPageError::img_err(url, e))
        })
        .collect::<Vec<_>>();

    join_all(futs).await.into_iter().collect()
}

