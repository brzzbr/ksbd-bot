use crate::domain::ksbd_page::KsbdPage;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, Default, Clone)]
pub struct PagesState {
    pages: Vec<KsbdPage>,
}

const INITIAL_PAGE: &str =
    "https://killsixbilliondemons.com/comic/kill-six-billion-demons-chapter-1/";

impl PagesState {
    pub fn start_from(&self) -> Option<(usize, String)> {
        match self.pages.len() {
            0 => Some((0, INITIAL_PAGE.to_string())),
            l => self
                .pages
                .last()
                .and_then(|p| p.next.as_ref())
                .map(|n| (l, n.to_string())),
        }
    }

    pub fn add_page(&mut self, page: KsbdPage) {
        if let Some(p) = self.pages.pop() {
            let prev_last = KsbdPage {
                idx: p.idx,
                title: p.title,
                url: p.url,
                imgs: p.imgs,
                text: p.text,
                next: Some(page.url.clone()),
            };

            self.pages.push(prev_last);
        };

        self.pages.push(page);
    }

    pub fn add_pages(&mut self, pages: Vec<KsbdPage>) {
        for p in pages {
            self.add_page(p)
        }
    }

    pub fn first(&self) -> Option<&KsbdPage> {
        self.pages.get(0)
    }

    pub fn last(&self) -> Option<&KsbdPage> {
        self.pages.last()
    }

    pub fn by_idx(&self, idx: usize) -> Option<&KsbdPage> {
        self.pages.get(idx)
    }
}

impl FromStr for PagesState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pages = s
            .split('\n')
            .enumerate()
            .map(|(idx, l)| {
                let l_split = l.split('\t').collect::<Vec<_>>();
                KsbdPage {
                    idx,
                    title: l_split[0].to_string(),
                    url: l_split[1].to_string(),
                    imgs: l_split[2]
                        .split('|')
                        .map(|u| u.to_string())
                        .collect::<Vec<_>>(),
                    text: l_split[3].to_string(),
                    next: if l_split[4] == "NO" {
                        None
                    } else {
                        Some(l_split[4].to_string())
                    },
                }
            })
            .collect::<Vec<_>>();

        Ok(PagesState { pages })
    }
}

impl ToString for PagesState {
    fn to_string(&self) -> String {
        self.pages
            .iter()
            .map(|p| {
                format!(
                    "{}\t{}\t{}\t{}\t{}",
                    p.title,
                    p.url,
                    p.imgs.join("|"),
                    p.text,
                    p.next.clone().unwrap_or("NO".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
