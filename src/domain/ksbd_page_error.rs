use std::fmt;

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
