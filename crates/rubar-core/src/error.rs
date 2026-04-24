use thiserror::Error;

#[derive(Error, Debug)]
pub enum RubarError {
    #[error("Invalid character '{char}' for {symbology}")]
    InvalidCharacter { char: char, symbology: String },

    #[error("Invalid length for {symbology}: expected {expected}, got {actual}")]
    InvalidLength {
        symbology: String,
        expected: usize,
        actual: usize,
    },

    #[error("Invalid checksum for {symbology}")]
    InvalidChecksum { symbology: String },

    #[error("Invalid unit '{0}': expected 'in', 'mm', or 'px'")]
    InvalidUnit(String),

    #[error("dpi is required when unit is 'in' or 'mm'")]
    DpiRequired,

    #[error("dpi must not be specified when unit is 'px'")]
    DpiForbidden,

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),

    #[error("Invalid GS1-128 value: {0}")]
    InvalidGs1(String),
}

pub type Result<T> = std::result::Result<T, RubarError>;
