use crate::error::{Result, RubarError};
use crate::geometry::{Bar, LinearGeometry};
use crate::symbol::Code128Symbol;
use barcoders::sym::code128::Code128;

// Unicode characters for Code128 character set switching (per barcoders docs)
const CHARSET_A: char = '\u{00C0}'; // À - Start/switch to character-set A
const CHARSET_B: char = '\u{0181}'; // Ɓ - Start/switch to character-set B
const CHARSET_C: char = '\u{0106}'; // Ć - Start/switch to character-set C

// Function codes (per barcoders docs)
const FNC1: char = '\u{0179}'; // Ź
const FNC2: char = '\u{017A}'; // ź
const FNC3: char = '\u{017B}'; // Ż
const FNC4: char = '\u{017C}'; // ż

/// Encode Code 128 symbols into linear geometry
pub fn encode_code128(symbols: &[Code128Symbol]) -> Result<LinearGeometry> {
    let mut data = String::new();
    let mut has_start = false;

    for symbol in symbols {
        match symbol {
            Code128Symbol::StartA => {
                if !has_start {
                    data.push(CHARSET_A);
                    has_start = true;
                }
            }
            Code128Symbol::StartB => {
                if !has_start {
                    data.push(CHARSET_B);
                    has_start = true;
                }
            }
            Code128Symbol::StartC => {
                if !has_start {
                    data.push(CHARSET_C);
                    has_start = true;
                }
            }
            Code128Symbol::Data(s) => {
                data.push_str(s);
            }
            Code128Symbol::FNC1 => {
                data.push(FNC1);
            }
            Code128Symbol::FNC2 => {
                data.push(FNC2);
            }
            Code128Symbol::FNC3 => {
                data.push(FNC3);
            }
            Code128Symbol::FNC4 => {
                data.push(FNC4);
            }
        }
    }

    // If no start symbol was specified, default to Code B (most common for alphanumeric)
    if !has_start {
        data.insert(0, CHARSET_B);
    }

    // Note: barcoders accepts empty data with just a start symbol, producing
    // a minimal barcode with start, checksum, and stop. We allow this since
    // it's technically valid Code 128.

    let barcode = Code128::new(&data)
        .map_err(|e| RubarError::EncodingError(e.to_string()))?;
    let encoded = barcode.encode();

    // Convert binary encoding to bars
    let bars = binary_to_bars(&encoded);
    let total_modules = encoded.len() as u32;

    Ok(LinearGeometry {
        bars,
        total_modules,
    })
}

/// Convert a binary slice (0s and 1s) to Bar structures
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
    fn test_code128_basic() {
        let geom = encode_code128(&[Code128Symbol::Data("HELLO".to_string())]).unwrap();
        assert!(geom.total_modules > 0);
        assert!(!geom.bars.is_empty());
    }

    #[test]
    fn test_code128_hello_geometry() {
        let geom = encode_code128(&[Code128Symbol::Data("HELLO".to_string())]).unwrap();
        // Code128 with Code B start for "HELLO":
        // Start B (11) + 5 chars (11 each) + checksum (11) + stop (13) = 90 modules
        assert_eq!(geom.total_modules, 90);
    }

    #[test]
    fn test_code128_with_fnc1() {
        let geom = encode_code128(&[
            Code128Symbol::FNC1,
            Code128Symbol::Data("01012345678901".to_string()),
        ])
        .unwrap();
        assert!(geom.total_modules > 0);
        assert!(!geom.bars.is_empty());
    }

    #[test]
    fn test_code128_explicit_start_c() {
        // Numeric data with explicit Code C start
        let geom = encode_code128(&[
            Code128Symbol::StartC,
            Code128Symbol::Data("123456".to_string()),
        ])
        .unwrap();
        assert!(geom.total_modules > 0);
    }

    #[test]
    fn test_code128_empty_data() {
        // Empty creates a minimal barcode with just start/checksum/stop
        let geom = encode_code128(&[]).unwrap();
        // Start B (11) + checksum (11) + stop (13) = 35 modules
        assert_eq!(geom.total_modules, 35);
    }

    #[test]
    fn test_code128_explicit_start_a() {
        let geom = encode_code128(&[
            Code128Symbol::StartA,
            Code128Symbol::Data("HELLO".to_string()),
        ])
        .unwrap();
        assert!(geom.total_modules > 0);
    }

    #[test]
    fn test_code128_gs1() {
        // GS1-128 format with FNC1 and application identifier
        let geom = encode_code128(&[
            Code128Symbol::StartC,
            Code128Symbol::FNC1,
            Code128Symbol::Data("01".to_string()),
            Code128Symbol::Data("12345678901234".to_string()),
        ])
        .unwrap();
        assert!(geom.total_modules > 0);
    }
}
