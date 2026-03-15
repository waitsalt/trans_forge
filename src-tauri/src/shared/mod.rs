pub mod actor;
pub mod command;
pub mod common;
pub mod database;
pub mod error;
pub mod state;

pub mod util {
    use tracing_subscriber::EnvFilter;

    pub fn log_init() {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        let _ = tracing_subscriber::fmt()
            .with_file(true)
            .with_line_number(true)
            .with_env_filter(env_filter)
            .try_init();
    }
}
