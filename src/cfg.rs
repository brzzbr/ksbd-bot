use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref DATA_PATH: String = env::var("DATA_PATH").unwrap();
}
