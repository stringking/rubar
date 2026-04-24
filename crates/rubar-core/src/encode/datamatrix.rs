use crate::error::{Result, RubarError};
use crate::geometry::MatrixGeometry;
use datamatrix::{DataMatrix, SymbolList};

/// Encode data as a Data Matrix (ECC 200).
///
/// When `gs1` is true, the encoder emits the FNC1 designator as the first
/// codeword so scanners recognize the output as GS1 Data Matrix. For GS1
/// payloads, variable-length Application Identifiers must be separated by
/// `\x1D` (ASCII Group Separator) in `data`. See [`crate::gs1::to_datamatrix_bytes`]
/// for a helper that produces a correctly-formatted payload from parsed AI
/// fields.
pub fn encode_datamatrix(data: &[u8], gs1: bool) -> Result<MatrixGeometry> {
    let symbol_list = SymbolList::default();
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
        let geom = encode_datamatrix(b"HELLO", false).unwrap();
        // Smallest square Data Matrix that fits "HELLO" is 10x10.
        assert!(geom.width >= 10);
        assert_eq!(geom.width, geom.height); // default symbol list picks square
        assert_eq!(geom.modules.len(), geom.height as usize);
        assert_eq!(geom.modules[0].len(), geom.width as usize);
    }

    #[test]
    fn encodes_gs1() {
        // GS1 Data Matrix with AI (01) GTIN + AI (17) expiry (both fixed-length)
        let payload = b"0112345678901234171231";
        let geom = encode_datamatrix(payload, true).unwrap();
        assert!(geom.width >= 10);
    }

    #[test]
    fn encodes_gs1_with_group_separators() {
        // Variable-length AI followed by another AI: AIs are separated by \x1D.
        let payload = b"10BATCH123\x1D0112345678901234";
        let geom = encode_datamatrix(payload, true).unwrap();
        assert!(geom.width >= 10);
    }

    #[test]
    fn finds_a_dark_module() {
        // Every valid Data Matrix has at least one dark module in the L finder
        // pattern (left + bottom edges). Quick sanity check that we populated
        // the module matrix at all.
        let geom = encode_datamatrix(b"X", false).unwrap();
        let any_dark = geom.modules.iter().flat_map(|r| r.iter()).any(|&b| b);
        assert!(any_dark);
    }
}
