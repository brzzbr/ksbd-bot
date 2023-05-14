pub mod ksbd_page;
pub mod pages_state;
pub mod subs_state;
pub mod bot_state;
pub mod bot_flow;
pub mod page_sender;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
