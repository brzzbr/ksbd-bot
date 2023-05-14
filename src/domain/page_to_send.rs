use crate::cfg::DATA_PATH;
use crate::domain::ksbd_page::KsbdPage;

pub struct PageToSend {
    pub idx: usize,
    pub title: String,
    pub imgs: Vec<String>,
    pub text: String,
    pub is_new: bool,
    pub has_next: bool,
}

impl PageToSend {
    fn new(p: KsbdPage, is_new: bool) -> PageToSend {
        PageToSend {
            idx: p.idx,
            title: p.title.clone(),
            imgs: p.imgs.clone(),
            text: p.text.clone(),
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
