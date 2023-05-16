use crate::cfg::DATA_PATH;
use crate::domain::ksbd_page::KsbdPage;

#[derive(Debug)]
pub struct PageToSend {
    pub idx: usize,
    pub title: Option<String>,
    pub imgs: Vec<String>,
    pub text: Vec<String>,
    pub is_new: bool,
    pub has_next: bool,
}

impl PageToSend {
    fn new(p: KsbdPage, is_new: bool) -> PageToSend {
        let raw_title = p
            .title
            .replace("%09", "\t")
            .replace("%0D%0A", "\n")
            .trim()
            .to_string();

        let text_blocks = p
            .text
            .replace("%09", "\t")
            .split("%0D%0A")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        // .replace("%0D%0A", "\n")
        // .trim()
        // .to_string();

        PageToSend {
            idx: p.idx,
            title: if raw_title.is_empty() {
                None
            } else {
                Some(raw_title)
            },
            imgs: p.imgs.clone(),
            text: text_blocks,
            is_new,
            has_next: p.next.is_some(),
        }
    }

    pub fn fresh_page(p: KsbdPage) -> PageToSend {
        PageToSend::new(p, true)
    }

    pub fn old_page(p: KsbdPage) -> PageToSend {
        PageToSend::new(p, false)
    }

    pub fn img_files(&self) -> Vec<String> {
        self.imgs
            .iter()
            .enumerate()
            .map(|(img_idx, _)| format!("{}/{}-{}.png", DATA_PATH.as_str(), self.idx, img_idx))
            .collect::<Vec<_>>()
    }
}
