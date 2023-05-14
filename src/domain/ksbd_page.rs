use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct KsbdPage {
    pub idx: usize,
    pub title: String,
    pub url: String,
    pub imgs: Vec<String>,
    pub text: String,
    pub next: Option<String>,
}

impl Display for KsbdPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KSBD Page [idx: {}, title: {}, url: {}, next: {}]",
            self.idx,
            self.title,
            self.url,
            self.next.to_owned().unwrap_or("NO YET".to_string())
        )
    }
}
