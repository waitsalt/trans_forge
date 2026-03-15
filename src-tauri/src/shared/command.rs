pub type CommandResult<T> = Result<T, String>;

pub fn to_client_error(error: impl std::fmt::Display) -> String {
    let message = error.to_string();
    tracing::error!("{message}");
    message
}
