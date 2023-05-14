use std::path;
use std::str::FromStr;

use lazy_static::lazy_static;
use tokio::fs;

use crate::cfg::DATA_PATH;
use crate::domain::subs_state::SubsState;

lazy_static! {
    static ref STATE_PATH: String = format!("{}/subs_state.txt", DATA_PATH.as_str());
}

pub async fn load_subs_state() -> SubsState {
    match path::Path::new(STATE_PATH.as_str()).exists() {
        false => SubsState::default(),
        true => fs::read_to_string(STATE_PATH.as_str())
            .await
            .map_err(|e| e.to_string())
            .and_then(|s| SubsState::from_str(s.as_str()))
            .unwrap(),
    }
}

pub async fn save_subs_state(state: &SubsState) {
    fs::write(STATE_PATH.as_str(), state.to_string())
        .await
        .unwrap()
}
