use crate::error::{Result, RubarError};
use crate::geometry::{Bar, LinearGeometry};
use barcoders::sym::ean13::UPCA;

/// Encode UPC-A barcode (11 or 12 digits)
pub fn encode_upc_a(data: &str) -> Result<LinearGeometry> {
    // Validate length
    if data.len() != 11 && data.len() != 12 {
        return Err(RubarError::InvalidLength {
            symbology: "UPC-A".to_string(),
            expected: 11,
            actual: data.len(),
        });
    }

    // Validate all digits
    for c in data.chars() {
        if !c.is_ascii_digit() {
            return Err(RubarError::InvalidCharacter {
                char: c,
                symbology: "UPC-A".to_string(),
            });
        }
    }

    // If 12 digits provided, verify checksum
    if data.len() == 12 {
        let calculated = calculate_upc_checksum(&data[..11]);
        let provided = data.chars().last().unwrap().to_digit(10).unwrap() as u8;
        if calculated != provided {
            return Err(RubarError::InvalidChecksum {
                symbology: "UPC-A".to_string(),
            });
        }
    }

    // UPC-A is essentially EAN-13 with a leading 0
    // barcoders' UPCA expects EAN-13 format (12-13 digits)
    // So we prepend a 0 to convert UPC-A to EAN-13
    let ean13_data = format!("0{}", data);

    let barcode =
        UPCA::new(&ean13_data).map_err(|e| RubarError::EncodingError(e.to_string()))?;
    let encoded = barcode.encode();

    let bars = binary_to_bars(&encoded);
    let total_modules = encoded.len() as u32;

    Ok(LinearGeometry {
        bars,
        total_modules,
    })
}

/// Calculate UPC-A check digit
fn calculate_upc_checksum(data: &str) -> u8 {
    let digits: Vec<u32> = data.chars().map(|c| c.to_digit(10).unwrap()).collect();

    let odd_sum: u32 = digits.iter().step_by(2).sum();
    let even_sum: u32 = digits.iter().skip(1).step_by(2).sum();

    let total = odd_sum * 3 + even_sum;
    let remainder = total % 10;

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
    fn test_upc_a_11_digits() {
        let geom = encode_upc_a("01234567890").unwrap();
        // UPC-A is 95 modules: 3 (start) + 42 (left) + 5 (center) + 42 (right) + 3 (end)
        assert_eq!(geom.total_modules, 95);
    }

    #[test]
    fn test_upc_a_12_digits_valid_checksum() {
        // 012345678905 has valid checksum
        let geom = encode_upc_a("012345678905").unwrap();
        assert_eq!(geom.total_modules, 95);
    }

    #[test]
    fn test_upc_a_invalid_checksum() {
        let result = encode_upc_a("012345678901");
        assert!(result.is_err());
    }

    #[test]
    fn test_upc_a_invalid_length() {
        let result = encode_upc_a("12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_upc_a_non_digit() {
        let result = encode_upc_a("0123456789A");
        assert!(result.is_err());
    }
}
