use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid file type")]
    InvalidFileType,
    #[error("Failed to decompress")]
    DecompressionError,
    #[error("Failed to unprotect")]
    UnprotectError,
    #[error("Misc. I/O error")]
    IoError(std::io::Error),
    #[error("Misc. nom error")]
    NomError,
}
