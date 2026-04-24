use crate::error::{Result, RubarError};
use crate::geometry::{Bar, LinearGeometry};
use barcoders::sym::code39::Code39;

/// Valid Code 39 characters
const VALID_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 -.$/+%";

/// Encode Code 39 barcode
pub fn encode_code39(data: &str) -> Result<LinearGeometry> {
    // Validate characters
    let upper = data.to_uppercase();
    for c in upper.chars() {
        if !VALID_CHARS.contains(c) {
            return Err(RubarError::InvalidCharacter {
                char: c,
                symbology: "Code 39".to_string(),
            });
        }
    }

    let barcode =
        Code39::new(&upper).map_err(|e| RubarError::EncodingError(e.to_string()))?;
    let encoded = barcode.encode();

    let bars = binary_to_bars(&encoded);
    let total_modules = encoded.len() as u32;

    Ok(LinearGeometry {
        bars,
        total_modules,
    })
}

fn binary_to_bars(encoded: &[u8]) -> Vec<Bar> {
    let mut bars = Vec::new();
    let mut i = 0;

    while i < encoded.len() {
        if encoded[i] == 1 {
            let start = i as u32;
            let mut width = 0u32;

            while i < encoded.len() && encoded[i] == 1 {
                width += 1;
                i += 1;
            }

            bars.push(Bar { x: start, width });
        } else {
            i += 1;
        }
    }

    bars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code39_basic() {
        let geom = encode_code39("HELLO").unwrap();
        assert!(geom.total_modules > 0);
        assert!(!geom.bars.is_empty());
    }

    #[test]
    fn test_code39_lowercase_converted() {
        let geom = encode_code39("hello").unwrap();
        assert!(geom.total_modules > 0);
    }

    #[test]
    fn test_code39_with_numbers() {
        let geom = encode_code39("ABC123").unwrap();
        assert!(geom.total_modules > 0);
    }

    #[test]
    fn test_code39_special_chars() {
        let geom = encode_code39("A-B.C").unwrap();
        assert!(geom.total_modules > 0);
    }

    #[test]
    fn test_code39_invalid_char() {
        let result = encode_code39("ABC@123");
        assert!(result.is_err());
    }
}
