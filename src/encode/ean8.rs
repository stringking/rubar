use crate::error::{Result, RubarError};
use crate::geometry::{Bar, LinearGeometry};
use barcoders::sym::ean8::EAN8;

/// Encode EAN-8 barcode (7 or 8 digits)
pub fn encode_ean8(data: &str) -> Result<LinearGeometry> {
    // Validate length
    if data.len() != 7 && data.len() != 8 {
        return Err(RubarError::InvalidLength {
            symbology: "EAN-8".to_string(),
            expected: 7,
            actual: data.len(),
        });
    }

    // Validate all digits
    for c in data.chars() {
        if !c.is_ascii_digit() {
            return Err(RubarError::InvalidCharacter {
                char: c,
                symbology: "EAN-8".to_string(),
            });
        }
    }

    // If 8 digits provided, verify checksum
    if data.len() == 8 {
        let calculated = calculate_ean8_checksum(&data[..7]);
        let provided = data.chars().last().unwrap().to_digit(10).unwrap() as u8;
        if calculated != provided {
            return Err(RubarError::InvalidChecksum {
                symbology: "EAN-8".to_string(),
            });
        }
    }

    let barcode =
        EAN8::new(data).map_err(|e| RubarError::EncodingError(e.to_string()))?;
    let encoded = barcode.encode();

    let bars = binary_to_bars(&encoded);
    let total_modules = encoded.len() as u32;

    Ok(LinearGeometry {
        bars,
        total_modules,
    })
}

/// Calculate EAN-8 check digit
fn calculate_ean8_checksum(data: &str) -> u8 {
    let digits: Vec<u32> = data.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // EAN-8: positions 1,3,5,7 (odd) multiplied by 3, positions 2,4,6 (even) by 1
    let weighted_sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 0 { d * 3 } else { d })
        .sum();

    let remainder = weighted_sum % 10;

    if remainder == 0 {
        0
    } else {
        (10 - remainder) as u8
    }
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
    fn test_ean8_7_digits() {
        let geom = encode_ean8("1234567").unwrap();
        // EAN-8 is 67 modules: 3 (start) + 28 (left) + 5 (center) + 28 (right) + 3 (end)
        assert_eq!(geom.total_modules, 67);
    }

    #[test]
    fn test_ean8_8_digits_valid_checksum() {
        // 12345670 has valid checksum
        let geom = encode_ean8("12345670").unwrap();
        assert_eq!(geom.total_modules, 67);
    }

    #[test]
    fn test_ean8_invalid_checksum() {
        let result = encode_ean8("12345671");
        assert!(result.is_err());
    }

    #[test]
    fn test_ean8_invalid_length() {
        let result = encode_ean8("12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_ean8_non_digit() {
        let result = encode_ean8("123456A");
        assert!(result.is_err());
    }
}
