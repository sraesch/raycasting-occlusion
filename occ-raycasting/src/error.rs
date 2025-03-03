use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CAD import error: {0}")]
    CadImport(#[from] cad_import::Error),

    #[error("File has either no extension or an invalid extension")]
    InvalidFileExtension,

    #[error("No loader found for the given file")]
    NoLoaderFound,
}

pub type Result<T> = std::result::Result<T, Error>;
