use thiserror::Error;

#[derive(Error, Debug)]
pub enum KodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Buffer error: {0}")]
    Buffer(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("LSP error: {0}")]
    Lsp(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("{0}")]
    Other(String),
}

pub type KodeResult<T> = Result<T, KodeError>;
