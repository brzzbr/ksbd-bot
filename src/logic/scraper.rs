use std::pin::Pin;
use std::time::Instant;

use async_trait::async_trait;
use futures::future::join_all;
use futures::TryFutureExt;
use futures::{stream, Stream};
use lazy_static::lazy_static;
use scraper::{Html, Selector};

use crate::cfg::DATA_PATH;
use crate::domain::ksbd_page::KsbdPage;
use crate::domain::ksbd_page_error::GetPageError;

lazy_static! {
    static ref SELECTOR_IMG: Selector = Selector::parse("#comic img").unwrap();
    static ref SELECTOR_NEXT: Selector = Selector::parse("#sidebar-over-comic > div > table > tbody > tr > td.comic_navi_right > a.navi.comic-nav-next.navi-next").unwrap();
    static ref SELECTOR_ENTRY: Selector = Selector::parse(".entry p").unwrap();
}

#[async_trait]
pub trait KsbdScraper {
    async fn request_page(&self, idx: usize, url: &str) -> Result<KsbdPage, GetPageError>;
    async fn download_imgs(&self, page: &KsbdPage) -> Result<(), GetPageError>;
    fn pages_from(&self, idx: usize, url: String) -> Pin<Box<dyn Stream<Item = KsbdPage> + Send +'_>>;
}

#[derive(Clone)]
// default prod implementation
pub struct KsbdScraperImpl {}

#[async_trait]
impl KsbdScraper for KsbdScraperImpl {
    async fn request_page(&self, idx: usize, url: &str) -> Result<KsbdPage, GetPageError> {
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

    async fn download_imgs(&self, page: &KsbdPage) -> Result<(), GetPageError> {
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

    fn pages_from(
        &self,
        idx: usize,
        url: String,
    ) -> Pin<Box<dyn Stream<Item = KsbdPage> + Send + '_>> {
        async fn get_page(
            me_myself_and_i: &KsbdScraperImpl,
            idx: usize,
            maybe_url: Option<String>,
        ) -> Option<(KsbdPage, (usize, Option<String>))> {
            if let Some(url) = maybe_url {
                let start = Instant::now();
                let page = me_myself_and_i.request_page(idx, &url).await.unwrap();
                let next = page.next.clone();
                // it's side-effecting here downloading the page. but I don't care atm.
                // highly likely should decouple it in a future... haha
                me_myself_and_i.download_imgs(&page).await.unwrap();
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

        let res_stream = stream::unfold((idx, Some(url)), |(idx, maybe_url)| {
            get_page(self, idx, maybe_url)
        });

        Box::pin(res_stream)
    }
}
