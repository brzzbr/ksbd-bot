pub mod bot_flow;
pub mod bot_state;
pub mod page_sender;
pub mod pages_state;
pub mod scraper;
pub mod subs_state;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
