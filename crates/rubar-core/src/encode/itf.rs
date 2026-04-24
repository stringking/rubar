use crate::error::{Result, RubarError};
use crate::geometry::{Bar, LinearGeometry};
use barcoders::sym::tf::TF;

/// Encode ITF (Interleaved 2 of 5) barcode
/// Requires even-length numeric string
pub fn encode_itf(data: &str) -> Result<LinearGeometry> {
    // Validate all digits
    for c in data.chars() {
        if !c.is_ascii_digit() {
            return Err(RubarError::InvalidCharacter {
                char: c,
                symbology: "ITF".to_string(),
            });
        }
    }

    // ITF requires even length
    if data.len() % 2 != 0 {
        return Err(RubarError::EncodingError(
            "ITF requires even-length data".to_string(),
        ));
    }

    if data.is_empty() {
        return Err(RubarError::EncodingError(
            "ITF data cannot be empty".to_string(),
        ));
    }

    let barcode =
        TF::interleaved(data).map_err(|e| RubarError::EncodingError(e.to_string()))?;
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
    fn test_itf_basic() {
        let geom = encode_itf("12345678").unwrap();
        assert!(geom.total_modules > 0);
        assert!(!geom.bars.is_empty());
    }

    #[test]
    fn test_itf_14() {
        // ITF-14 is commonly used for shipping
        let geom = encode_itf("00012345678905").unwrap();
        assert!(geom.total_modules > 0);
    }

    #[test]
    fn test_itf_odd_length() {
        let result = encode_itf("1234567");
        assert!(result.is_err());
    }

    #[test]
    fn test_itf_non_digit() {
        let result = encode_itf("12A456");
        assert!(result.is_err());
    }

    #[test]
    fn test_itf_empty() {
        let result = encode_itf("");
        assert!(result.is_err());
    }
}
