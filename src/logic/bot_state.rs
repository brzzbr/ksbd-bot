use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use teloxide::prelude::ChatId;
use tokio::sync::RwLock;

use crate::domain::bot_state::BotState;
use crate::domain::ksbd_page::KsbdPage;
use crate::logic::pages_state::{PagesStateManager, PagesStateManagerImpl};
use crate::logic::scraper::KsbdScraper;
use crate::logic::subs_state::{SubsStateManager, SubsStateManagerImpl};

#[async_trait]
pub trait BotStateManager {
    async fn maybe_last(&self) -> Option<KsbdPage>;
    async fn subs_chat_ids(&self) -> Vec<ChatId>;

    async fn add_subs(&self, chat_id: ChatId);
    async fn add_pages(&self, pages: Vec<KsbdPage>);

    async fn first(&self) -> Option<KsbdPage>;
    async fn last(&self) -> Option<KsbdPage>;
    async fn by_idx(&self, idx: usize) -> Option<KsbdPage>;
}

#[async_trait]
pub trait BotStateManagerInit {
    async fn init(
        scraper: &(impl KsbdScraper + Send + Sync),
        // pages_state_manager: &impl PagesStateManager,
        // subs_state_manager: &impl SubsStateManager,
    ) -> Self;
}

#[async_trait]
impl BotStateManagerInit for BotStateManagerImpl {
    async fn init(
        scraper: &(impl KsbdScraper + Send + Sync),
        // pages_state_manager: &impl PagesStateManager,
        // subs_state_manager: &impl SubsStateManager,
    ) -> Self {
        let pages_state_manager = PagesStateManagerImpl {};
        let subs_state_manager = SubsStateManagerImpl {};

        let pages_state = pages_state_manager.load_pages_state().await;
        if let Some((idx, url)) = pages_state.start_from() {
            log::info!("restoring full state, from {}", idx);
            scraper
                .pages_from(idx, url)
                .fold(pages_state.clone(), move |mut state, page| async move {
                    state.add_page(page);
                    pages_state_manager.save_pages_state(&state).await;
                    state
                })
                .await;
            log::info!("state has been restored");
        }
        let pages_state = pages_state_manager.load_pages_state().await;
        let subs_state = subs_state_manager.load_subs_state().await;

        let inner_state = Arc::new(RwLock::new(BotState {
            pages: pages_state,
            subscribers: subs_state,
        }));

        let p_manager = Box::pin(pages_state_manager);
        let s_manager = Box::pin(subs_state_manager);

        BotStateManagerImpl {
            inner_state,
            pages_state_manager: p_manager,
            subs_state_manager: s_manager,
        }
    }
}

pub struct BotStateManagerImpl {
    inner_state: Arc<RwLock<BotState>>,
    pages_state_manager: Pin<Box<dyn PagesStateManager + Send + Sync>>,
    subs_state_manager: Pin<Box<dyn SubsStateManager + Send + Sync>>,
}

impl Clone for BotStateManagerImpl {
    fn clone(&self) -> Self {
        // screw it too
        let p_manager = Box::pin(PagesStateManagerImpl {});
        let s_manager = Box::pin(SubsStateManagerImpl {});

        BotStateManagerImpl {
            inner_state: self.inner_state.clone(),
            pages_state_manager: p_manager,
            subs_state_manager: s_manager,
        }
    }
}

#[async_trait]
impl BotStateManager for BotStateManagerImpl {
    async fn maybe_last(&self) -> Option<KsbdPage> {
        self.inner_state.read().await.pages.last().cloned()
    }

    async fn subs_chat_ids(&self) -> Vec<ChatId> {
        self.inner_state.read().await.subscribers.clone().chat_ids()
    }

    async fn add_subs(&self, chat_id: ChatId) {
        let mut state_to_write = self.inner_state.write().await;
        state_to_write.subscribers.add(chat_id.0);
        self.subs_state_manager
            .save_subs_state(&state_to_write.subscribers)
            .await
    }

    async fn add_pages(&self, pages: Vec<KsbdPage>) {
        let mut state_to_write = self.inner_state.write().await;
        state_to_write.pages.add_pages(pages.clone());
        self.pages_state_manager
            .save_pages_state(&state_to_write.pages)
            .await
    }

    async fn first(&self) -> Option<KsbdPage> {
        let state = self.inner_state.read().await;
        state.pages.first().cloned()
    }

    async fn last(&self) -> Option<KsbdPage> {
        let state = self.inner_state.read().await;
        state.pages.last().cloned()
    }

    async fn by_idx(&self, idx: usize) -> Option<KsbdPage> {
        let state = self.inner_state.read().await;
        state.pages.by_idx(idx).cloned()
    }
}
