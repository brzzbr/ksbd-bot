use std::path;
use std::str::FromStr;

use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::fs;

use crate::cfg::DATA_PATH;
use crate::domain::pages_state::PagesState;

lazy_static! {
    static ref STATE_PATH: String = format!("{}/pages_state.txt", DATA_PATH.as_str());
}

#[async_trait]
pub trait PagesStateManager {
    async fn load_pages_state(&self) -> PagesState;
    async fn save_pages_state(&self, state: &PagesState);
}

// prod implementation
#[derive(Clone, Copy)]
pub struct PagesStateManagerImpl {}

#[async_trait]
impl PagesStateManager for PagesStateManagerImpl {
    // if it fails, it fails!
    async fn load_pages_state(&self) -> PagesState {
        match path::Path::new(STATE_PATH.as_str()).exists() {
            false => PagesState::default(),
            true => fs::read_to_string(STATE_PATH.as_str())
                .await
                .map_err(|e| e.to_string())
                .and_then(|s| PagesState::from_str(s.as_str()))
                .unwrap(),
        }
    }

    // if it fails, it fails!
    async fn save_pages_state(&self, state: &PagesState) {
        fs::write(STATE_PATH.as_str(), state.to_string())
            .await
            .unwrap()
    }
}
