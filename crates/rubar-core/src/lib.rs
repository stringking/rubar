//! rubar-core: barcode encoding and rendering, pure Rust.
//!
//! This crate has no Python dependency. Consumers who want Python bindings
//! should use the `rubar-py` crate (and the `rubar` PyPI package).

pub mod encode;
pub mod error;
pub mod geometry;
pub mod gs1;
pub mod render;
pub mod symbol;
pub mod unit;

pub use encode::{
    encode_code128, encode_code39, encode_datamatrix, encode_ean8, encode_itf, encode_qr,
    encode_upc_a,
};
pub use error::{Result, RubarError};
pub use geometry::{Bar, LinearGeometry, MatrixGeometry};
pub use render::{render_linear_png, render_linear_svg, render_matrix_png, render_matrix_svg};
pub use symbol::Code128Symbol;
pub use unit::Unit;
