//! GS1-128 Application Identifier parsing.
//!
//! Parses the canonical parenthesized form — `(01)12345678901234(17)260101(10)BATCH123` —
//! into a list of `(AI, data)` pairs, and builds the corresponding Code 128 symbol
//! stream (Start-B, FNC1 designator, data, and FNC1 separators after
//! variable-length fields).

use crate::error::{Result, RubarError};
use crate::symbol::Code128Symbol;

/// A parsed GS1 Application Identifier and its associated data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiField {
    pub ai: String,
    pub data: String,
}

/// Parse a GS1-128 input string in parenthesized form.
pub fn parse(value: &str) -> Result<Vec<AiField>> {
    let mut fields = Vec::new();
    let mut chars = value.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c != '(' {
            return Err(RubarError::InvalidGs1(
                "value must begin with '(' and use (AI)data syntax".to_string(),
            ));
        }
        chars.next(); // consume '('

        let mut ai = String::new();
        let mut closed = false;
        for ch in chars.by_ref() {
            if ch == ')' {
                closed = true;
                break;
            }
            ai.push(ch);
        }
        if !closed {
            return Err(RubarError::InvalidGs1("unterminated AI: missing ')'".to_string()));
        }
        if ai.is_empty() || ai.len() > 4 || !ai.chars().all(|c| c.is_ascii_digit()) {
            return Err(RubarError::InvalidGs1(format!(
                "invalid Application Identifier '{}': must be 2-4 digits",
                ai
            )));
        }

        let mut data = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '(' {
                break;
            }
            data.push(ch);
            chars.next();
        }
        if data.is_empty() {
            return Err(RubarError::InvalidGs1(format!("AI ({}) has no data", ai)));
        }
        if let Some(expected) = fixed_ai_length(&ai) {
            if data.len() != expected {
                return Err(RubarError::InvalidGs1(format!(
                    "AI ({}) requires exactly {} characters of data, got {}",
                    ai,
                    expected,
                    data.len()
                )));
            }
        }

        fields.push(AiField { ai, data });
    }

    if fields.is_empty() {
        return Err(RubarError::InvalidGs1(
            "value must contain at least one (AI)data pair".to_string(),
        ));
    }

    Ok(fields)
}

/// Build a Code 128 symbol stream for a GS1-128 barcode.
///
/// Starts with `StartB` + `FNC1` (the GS1-128 designator), emits each AI+data
/// as a `Data` symbol, and inserts an `FNC1` between variable-length fields.
pub fn to_symbols(fields: &[AiField]) -> Vec<Code128Symbol> {
    let mut out = Vec::with_capacity(fields.len() * 2 + 2);
    out.push(Code128Symbol::StartB);
    out.push(Code128Symbol::FNC1);
    for (i, field) in fields.iter().enumerate() {
        let mut payload = String::with_capacity(field.ai.len() + field.data.len());
        payload.push_str(&field.ai);
        payload.push_str(&field.data);
        out.push(Code128Symbol::Data(payload));
        let is_last = i + 1 == fields.len();
        let is_variable = fixed_ai_length(&field.ai).is_none();
        if !is_last && is_variable {
            out.push(Code128Symbol::FNC1);
        }
    }
    out
}

/// Build the byte payload for a GS1 Data Matrix.
///
/// Concatenates each AI+data, inserting an ASCII Group Separator (`\x1D`)
/// between variable-length fields. The datamatrix crate's `encode_gs1` adds
/// the FNC1 designator codeword itself; callers should pass the return
/// value of this function straight into `encode_datamatrix(..., gs1=true)`.
pub fn to_datamatrix_bytes(fields: &[AiField]) -> Vec<u8> {
    const GS: u8 = 0x1D;
    let mut out = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        out.extend_from_slice(field.ai.as_bytes());
        out.extend_from_slice(field.data.as_bytes());
        let is_last = i + 1 == fields.len();
        let is_variable = fixed_ai_length(&field.ai).is_none();
        if !is_last && is_variable {
            out.push(GS);
        }
    }
    out
}

/// Canonical parenthesized human-readable representation of parsed fields.
pub fn format_human_readable(fields: &[AiField]) -> String {
    let mut out = String::new();
    for field in fields {
        out.push('(');
        out.push_str(&field.ai);
        out.push(')');
        out.push_str(&field.data);
    }
    out
}

/// Required data length for a fixed-length GS1 AI, or `None` for variable-length.
///
/// Sourced from the GS1 General Specifications. Lengths exclude the AI itself.
pub fn fixed_ai_length(ai: &str) -> Option<usize> {
    match ai {
        "00" => Some(18),
        "01" | "02" | "03" => Some(14),
        "04" => Some(16),
        "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" | "19" => Some(6),
        "20" => Some(2),
        "41" => Some(13),
        // 31xx-36xx: measurements, all 6 digits
        s if s.len() == 4
            && s.starts_with('3')
            && matches!(&s[1..2], "1" | "2" | "3" | "4" | "5" | "6")
            && s[2..].chars().all(|c| c.is_ascii_digit()) =>
        {
            Some(6)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_fixed_ai() {
        let fields = parse("(01)12345678901234").unwrap();
        assert_eq!(fields, vec![AiField { ai: "01".into(), data: "12345678901234".into() }]);
    }

    #[test]
    fn parses_multiple_ais() {
        let fields = parse("(01)12345678901234(17)260101(10)BATCH123").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].ai, "01");
        assert_eq!(fields[1].ai, "17");
        assert_eq!(fields[2].data, "BATCH123");
    }

    #[test]
    fn rejects_missing_open_paren() {
        assert!(parse("01)12345678901234").is_err());
    }

    #[test]
    fn rejects_unterminated_ai() {
        assert!(parse("(0112345678901234").is_err());
    }

    #[test]
    fn rejects_non_numeric_ai() {
        assert!(parse("(AB)foo").is_err());
    }

    #[test]
    fn rejects_empty_data() {
        assert!(parse("(01)").is_err());
    }

    #[test]
    fn rejects_wrong_length_for_fixed_ai() {
        assert!(parse("(01)12345").is_err());
    }

    #[test]
    fn fixed_ai_lookup() {
        assert_eq!(fixed_ai_length("01"), Some(14));
        assert_eq!(fixed_ai_length("17"), Some(6));
        assert_eq!(fixed_ai_length("3103"), Some(6));
        assert_eq!(fixed_ai_length("3920"), None);
        assert_eq!(fixed_ai_length("10"), None);
        assert_eq!(fixed_ai_length("21"), None);
    }

    #[test]
    fn to_symbols_starts_with_fnc1_designator() {
        let fields = parse("(01)12345678901234").unwrap();
        let symbols = to_symbols(&fields);
        assert_eq!(symbols[0], Code128Symbol::StartB);
        assert_eq!(symbols[1], Code128Symbol::FNC1);
    }

    #[test]
    fn to_symbols_no_separator_after_fixed() {
        let fields = parse("(01)12345678901234(17)260101").unwrap();
        let symbols = to_symbols(&fields);
        // StartB, FNC1, Data("0112345678901234"), Data("17260101")
        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[2], Code128Symbol::Data("0112345678901234".into()));
        assert_eq!(symbols[3], Code128Symbol::Data("17260101".into()));
    }

    #[test]
    fn to_symbols_separator_after_variable() {
        let fields = parse("(10)BATCH(01)12345678901234").unwrap();
        let symbols = to_symbols(&fields);
        // StartB, FNC1, Data("10BATCH"), FNC1, Data("0112345678901234")
        assert_eq!(symbols.len(), 5);
        assert_eq!(symbols[2], Code128Symbol::Data("10BATCH".into()));
        assert_eq!(symbols[3], Code128Symbol::FNC1);
        assert_eq!(symbols[4], Code128Symbol::Data("0112345678901234".into()));
    }

    #[test]
    fn to_symbols_no_trailing_fnc1() {
        let fields = parse("(01)12345678901234(10)BATCH").unwrap();
        let symbols = to_symbols(&fields);
        // Last symbol must not be an FNC1 (terminator) — the variable-length
        // 10BATCH is the last field, so no separator.
        assert_ne!(symbols.last(), Some(&Code128Symbol::FNC1));
    }

    #[test]
    fn datamatrix_bytes_no_separator_after_fixed() {
        // Both AIs are fixed-length, so no \x1D should appear between them.
        let fields = parse("(01)12345678901234(17)260101").unwrap();
        let bytes = to_datamatrix_bytes(&fields);
        assert_eq!(bytes, b"011234567890123417260101");
    }

    #[test]
    fn datamatrix_bytes_separator_after_variable() {
        let fields = parse("(10)BATCH(01)12345678901234").unwrap();
        let bytes = to_datamatrix_bytes(&fields);
        // 10BATCH <GS> 0112345678901234
        assert_eq!(bytes[..7], *b"10BATCH");
        assert_eq!(bytes[7], 0x1D);
        assert_eq!(&bytes[8..], b"0112345678901234");
    }

    #[test]
    fn datamatrix_bytes_no_trailing_separator() {
        // Last field is variable-length — no trailing GS
        let fields = parse("(01)12345678901234(10)BATCH").unwrap();
        let bytes = to_datamatrix_bytes(&fields);
        assert_ne!(*bytes.last().unwrap(), 0x1D);
    }

    #[test]
    fn format_human_readable_roundtrips() {
        let raw = "(01)12345678901234(10)BATCH123";
        let fields = parse(raw).unwrap();
        assert_eq!(format_human_readable(&fields), raw);
    }
}
