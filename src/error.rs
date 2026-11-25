use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read configuration file")]
    Read,
    #[error("configuration is not valid TOML")]
    Syntax,
    #[error("configuration contains invalid or unknown fields")]
    Fields,
    #[error("environment variable {0} is required by configuration")]
    MissingEnv(String),
    #[error("{0}")]
    Validation(String),
}
