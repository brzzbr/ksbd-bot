use crate::domain::pages_state::PagesState;
use crate::domain::subs_state::SubsState;

pub struct BotState {
    pub pages: PagesState,
    pub subscribers: SubsState,
}
