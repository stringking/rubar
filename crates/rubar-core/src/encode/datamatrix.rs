use crate::error::{Result, RubarError};
use crate::geometry::MatrixGeometry;
use datamatrix::{DataMatrix, SymbolList};

/// Shape constraint for Data Matrix encoding.
///
/// The Data Matrix (ECC 200) spec defines both square sizes (10x10 through
/// 144x144) and rectangular sizes (8x18, 8x32, 12x26, etc.). For any given
/// payload the encoder picks the smallest-area symbol that fits; if you
/// want a specific shape regardless of area, constrain the symbol list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DataMatrixShape {
    /// Allow any shape; the encoder picks the smallest-area symbol. Default.
    #[default]
    Any,
    /// Restrict to square symbols (10x10, 12x12, ..., 144x144).
    Square,
    /// Restrict to rectangular symbols (8x18, 8x32, 12x26, 12x36, 16x36, 16x48).
    Rectangular,
}

impl DataMatrixShape {
    fn apply(self, list: SymbolList) -> SymbolList {
        match self {
            DataMatrixShape::Any => list,
            DataMatrixShape::Square => list.enforce_square(),
            DataMatrixShape::Rectangular => list.enforce_rectangular(),
        }
    }
}

/// Encode data as a Data Matrix (ECC 200).
///
/// When `gs1` is true, the encoder emits the FNC1 designator as the first
/// codeword so scanners recognize the output as GS1 Data Matrix. For GS1
/// payloads, variable-length Application Identifiers must be separated by
/// `\x1D` (ASCII Group Separator) in `data`. See [`crate::gs1::to_datamatrix_bytes`]
/// for a helper that produces a correctly-formatted payload from parsed AI
/// fields.
///
/// `shape` controls whether the encoder picks a square, rectangular, or any
/// symbol size. Both GS1 and plain Data Matrix support both shapes per ISO
/// 16022 / GS1 General Specifications.
pub fn encode_datamatrix(data: &[u8], gs1: bool, shape: DataMatrixShape) -> Result<MatrixGeometry> {
    let symbol_list = shape.apply(SymbolList::default());
    if symbol_list.is_empty() {
        return Err(RubarError::EncodingError(
            "symbol list is empty for requested shape".to_string(),
        ));
    }
    let code = if gs1 {
        DataMatrix::encode_gs1(data, symbol_list)
    } else {
        DataMatrix::encode(data, symbol_list)
    }
    .map_err(|e| RubarError::EncodingError(format!("Data Matrix encoding failed: {:?}", e)))?;

    let bitmap = code.bitmap();
    let width = bitmap.width() as u32;
    let height = bitmap.height() as u32;

    // Build the [row][col] bool matrix consumers expect.
    let bits = bitmap.bits();
    let mut modules = Vec::with_capacity(height as usize);
    for y in 0..height as usize {
        let start = y * width as usize;
        let end = start + width as usize;
        modules.push(bits[start..end].to_vec());
    }

    Ok(MatrixGeometry {
        modules,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_ascii_data() {
        let geom = encode_datamatrix(b"HELLO", false, DataMatrixShape::Any).unwrap();
        // Smallest symbol that fits "HELLO" is 10x10 (square) or a rectangle.
        assert!(geom.width >= 8);
        assert_eq!(geom.modules.len(), geom.height as usize);
        assert_eq!(geom.modules[0].len(), geom.width as usize);
    }

    #[test]
    fn encodes_gs1() {
        let payload = b"0112345678901234171231";
        let geom = encode_datamatrix(payload, true, DataMatrixShape::Any).unwrap();
        assert!(geom.width >= 8);
    }

    #[test]
    fn encodes_gs1_with_group_separators() {
        // Variable-length AI followed by another AI: AIs are separated by \x1D.
        let payload = b"10BATCH123\x1D0112345678901234";
        let geom = encode_datamatrix(payload, true, DataMatrixShape::Any).unwrap();
        assert!(geom.width >= 8);
    }

    #[test]
    fn square_shape_forces_square_output() {
        // Payload that would otherwise pick rectangular (smaller area).
        let payload = b"10BATCH\x1D0112345678901234";
        let geom = encode_datamatrix(payload, true, DataMatrixShape::Square).unwrap();
        assert!(geom.is_square(), "expected square, got {}x{}", geom.width, geom.height);
    }

    #[test]
    fn rectangular_shape_forces_rect_output() {
        // Small payload that would normally pick square 10x10; force rectangular.
        let geom = encode_datamatrix(b"HI", false, DataMatrixShape::Rectangular).unwrap();
        assert!(!geom.is_square(), "expected rectangular, got {}x{}", geom.width, geom.height);
    }

    #[test]
    fn finds_a_dark_module() {
        let geom = encode_datamatrix(b"X", false, DataMatrixShape::Any).unwrap();
        let any_dark = geom.modules.iter().flat_map(|r| r.iter()).any(|&b| b);
        assert!(any_dark);
    }
}
