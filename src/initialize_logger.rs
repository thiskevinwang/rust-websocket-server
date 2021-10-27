use std::env;

// static RUST_LOG: &str = "RUST_LOG";
const RUST_LOG: &'static str = "RUST_LOG";

pub fn initialize_logger(level: log::Level) -> () {
    // Set env var
    env::set_var(RUST_LOG, level.to_string());
    // init logger
    pretty_env_logger::init();
    // [trace|debug|info|warn|error]!("test")
}
