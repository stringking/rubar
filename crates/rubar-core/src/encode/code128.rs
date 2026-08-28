//! Code 128 encoding, with minimal-length code-set planning.
//!
//! Code 128 has three code sets. A and B carry one character per symbol; C
//! carries *two digits* per symbol. Encoding a digit run in Code C therefore
//! nearly halves the symbol count, and since callers size a barcode from its
//! symbol count, encoding everything in Code B silently produces a barcode up
//! to 1.76x denser than the caller sized it for.
//!
//! So this module plans: a dynamic program over the token stream picks the
//! start code set and the latch points that minimise the total symbol count.
//! The result is handed to `barcoders`, which wants the code-set switches
//! in-band as the Unicode markers below.

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

// barcoders spells DEL (ASCII 127) as ÷ in its Code B column.
const DEL_IN_CODE_B: char = '\u{00F7}';

/// Cost sentinel. `u32::MAX / 4` rather than `u32::MAX` so `INF + 1` in the
/// latch relaxation cannot overflow.
const INF: u32 = u32::MAX / 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeSet {
    A,
    B,
    C,
}

impl CodeSet {
    /// Tie-break order for choosing a start set: B first, so alphanumeric
    /// payloads keep the conventional Code B start when A would tie.
    const PREFERENCE: [CodeSet; 3] = [CodeSet::B, CodeSet::A, CodeSet::C];

    fn idx(self) -> usize {
        match self {
            CodeSet::A => 0,
            CodeSet::B => 1,
            CodeSet::C => 2,
        }
    }

    /// The in-band marker that starts in, or latches to, this set.
    fn marker(self) -> char {
        match self {
            CodeSet::A => CHARSET_A,
            CodeSet::B => CHARSET_B,
            CodeSet::C => CHARSET_C,
        }
    }
}

/// One planning unit: an ASCII data character, or a function character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    /// ASCII only — validated on the way in, so `0..=127`.
    Ch(u8),
    /// FNC1 through FNC4.
    Fnc(u8),
}

impl Token {
    fn is_digit(self) -> bool {
        matches!(self, Token::Ch(c) if c.is_ascii_digit())
    }

    /// Whether this token is a single symbol in code set A or B.
    ///
    /// Code A covers ASCII 0-95 (controls, space, digits, uppercase, symbols);
    /// Code B covers ASCII 32-127. All four function characters exist in both.
    fn fits(self, set: CodeSet) -> bool {
        match self {
            Token::Fnc(_) => matches!(set, CodeSet::A | CodeSet::B),
            Token::Ch(c) => match set {
                CodeSet::A => c < 96,
                CodeSet::B => c >= 32,
                CodeSet::C => false,
            },
        }
    }
}

/// Encode Code 128 symbols into linear geometry.
///
/// The code sets are chosen to minimise the symbol count — in particular digit
/// runs of four or more pack two-per-symbol in Code C. A `StartA`/`StartB`/
/// `StartC` symbol pins the *initial* code set; planning then proceeds from
/// there, so pinning never costs more than one extra symbol and usually costs
/// nothing. Only the first start symbol is honoured, whatever its position.
pub fn encode_code128(symbols: &[Code128Symbol]) -> Result<LinearGeometry> {
    let (tokens, pinned) = flatten(symbols)?;
    let data = plan(&tokens, pinned)?;

    // Note: barcoders accepts data with just a start symbol, producing
    // a minimal barcode with start, checksum, and stop. We allow this since
    // it's technically valid Code 128.

    let barcode = Code128::new(&data).map_err(|e| RubarError::EncodingError(e.to_string()))?;
    let encoded = barcode.encode();

    // Convert binary encoding to bars
    let bars = binary_to_bars(&encoded);
    let total_modules = encoded.len() as u32;

    Ok(LinearGeometry {
        bars,
        total_modules,
    })
}

/// Flatten the symbol list into planning tokens plus any pinned start set.
///
/// Data must be ASCII. Rejecting non-ASCII is not just hygiene: barcoders
/// takes its code-set switches in-band, so a literal `Ć` or `Ź` inside a
/// `Data` payload would otherwise be silently interpreted as a switch or
/// function character rather than encoded as data.
fn flatten(symbols: &[Code128Symbol]) -> Result<(Vec<Token>, Option<CodeSet>)> {
    let mut tokens = Vec::new();
    let mut pinned = None;

    for symbol in symbols {
        match symbol {
            Code128Symbol::StartA => pinned = pinned.or(Some(CodeSet::A)),
            Code128Symbol::StartB => pinned = pinned.or(Some(CodeSet::B)),
            Code128Symbol::StartC => pinned = pinned.or(Some(CodeSet::C)),
            Code128Symbol::FNC1 => tokens.push(Token::Fnc(1)),
            Code128Symbol::FNC2 => tokens.push(Token::Fnc(2)),
            Code128Symbol::FNC3 => tokens.push(Token::Fnc(3)),
            Code128Symbol::FNC4 => tokens.push(Token::Fnc(4)),
            Code128Symbol::Data(s) => {
                for ch in s.chars() {
                    if !ch.is_ascii() {
                        return Err(RubarError::InvalidCharacter {
                            char: ch,
                            symbology: "Code 128".to_string(),
                        });
                    }
                    tokens.push(Token::Ch(ch as u8));
                }
            }
        }
    }

    Ok((tokens, pinned))
}

/// `cost[i][s]` = minimal symbols to encode `tokens[i..]` while in code set
/// `s`, counting any latch emitted at `i`. Start, checksum and stop are
/// excluded — they are fixed overhead.
///
/// This mirrors sklib's `min_symbol_count` oracle with one deliberate
/// omission: no SHIFT. barcoders' `parse()` does not model shift state (the
/// shift marker resolves as an ordinary data symbol and leaves the current set
/// unchanged, so the character after it is looked up in the wrong set), which
/// makes SHIFT unreachable through this backend. SHIFT beats latch-there-and-
/// back by exactly one symbol, and only for payloads that *alternate* between
/// Code-A-only characters (ASCII < 32) and Code-B-only characters (ASCII >=
/// 96). Every printable-ASCII, GS1 and real-world payload plans identically.
fn cost_table(tokens: &[Token]) -> Vec<[u32; 3]> {
    let n = tokens.len();
    let mut table = vec![[INF; 3]; n + 1];
    table[n] = [0; 3];

    for i in (0..n).rev() {
        let mut cost = [INF; 3];
        for set in [CodeSet::A, CodeSet::B] {
            if tokens[i].fits(set) {
                cost[set.idx()] = 1 + table[i + 1][set.idx()];
            }
        }
        cost[CodeSet::C.idx()] = consume_in_c(tokens, &table, i);

        // Latch relaxation: entering set s we may spend one symbol latching to
        // another set and consume there instead. Two passes reach a fixpoint
        // across three sets.
        for _ in 0..2 {
            for s in 0..3 {
                for t in 0..3 {
                    if s != t && cost[t] + 1 < cost[s] {
                        cost[s] = cost[t] + 1;
                    }
                }
            }
        }

        table[i] = cost;
    }

    table
}

/// Cost of consuming `tokens[i..]` starting in Code C without latching first.
///
/// Code C holds digit *pairs*, so a digit only fits when the next token is
/// also a digit. FNC1 is the one function character available in Code C and
/// consumes a single symbol without disturbing pairing.
fn consume_in_c(tokens: &[Token], table: &[[u32; 3]], i: usize) -> u32 {
    if tokens[i] == Token::Fnc(1) {
        1 + table[i + 1][CodeSet::C.idx()]
    } else if tokens[i].is_digit() && tokens.get(i + 1).is_some_and(|t| t.is_digit()) {
        1 + table[i + 2][CodeSet::C.idx()]
    } else {
        INF
    }
}

/// Cost of consuming `tokens[i..]` in `set` without latching first.
fn consume(tokens: &[Token], table: &[[u32; 3]], i: usize, set: CodeSet) -> u32 {
    match set {
        CodeSet::C => consume_in_c(tokens, table, i),
        _ if tokens[i].fits(set) => 1 + table[i + 1][set.idx()],
        _ => INF,
    }
}

/// Build the in-band barcoders string for a minimal-length encoding.
fn plan(tokens: &[Token], pinned: Option<CodeSet>) -> Result<String> {
    let table = cost_table(tokens);

    let start = match pinned {
        Some(set) => set,
        None => *CodeSet::PREFERENCE
            .iter()
            .min_by_key(|set| table[0][set.idx()])
            .expect("PREFERENCE is non-empty"),
    };

    let mut out = String::with_capacity(tokens.len() + 4);
    let mut set = start;
    out.push(set.marker());

    let mut i = 0;
    while i < tokens.len() {
        if consume(tokens, &table, i, set) == table[i][set.idx()] {
            match set {
                CodeSet::C if tokens[i] != Token::Fnc(1) => {
                    // A digit pair: two characters, one symbol.
                    push_token(&mut out, tokens[i], set);
                    push_token(&mut out, tokens[i + 1], set);
                    i += 2;
                }
                _ => {
                    push_token(&mut out, tokens[i], set);
                    i += 1;
                }
            }
        } else {
            let target = [CodeSet::A, CodeSet::B, CodeSet::C]
                .into_iter()
                .find(|t| *t != set && table[i][t.idx()] + 1 == table[i][set.idx()])
                .ok_or_else(|| {
                    RubarError::EncodingError(
                        "Code 128 planner found no encoding for the payload".to_string(),
                    )
                })?;
            out.push(target.marker());
            set = target;
        }
    }

    Ok(out)
}

fn push_token(out: &mut String, token: Token, set: CodeSet) {
    match token {
        Token::Fnc(1) => out.push(FNC1),
        Token::Fnc(2) => out.push(FNC2),
        Token::Fnc(3) => out.push(FNC3),
        Token::Fnc(_) => out.push(FNC4),
        Token::Ch(127) if set == CodeSet::B => out.push(DEL_IN_CODE_B),
        Token::Ch(c) => out.push(c as char),
    }
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

    /// Build symbols from a corpus payload where `\x1d` marks an FNC1.
    fn symbols(payload: &str) -> Vec<Code128Symbol> {
        let mut out = Vec::new();
        let mut buf = String::new();
        for ch in payload.chars() {
            if ch == '\x1d' {
                if !buf.is_empty() {
                    out.push(Code128Symbol::Data(std::mem::take(&mut buf)));
                }
                out.push(Code128Symbol::FNC1);
            } else {
                buf.push(ch);
            }
        }
        if !buf.is_empty() {
            out.push(Code128Symbol::Data(buf));
        }
        out
    }

    fn modules(payload: &str) -> u32 {
        encode_code128(&symbols(payload)).unwrap().total_modules
    }

    #[test]
    fn test_code128_basic() {
        let geom = encode_code128(&[Code128Symbol::Data("HELLO".to_string())]).unwrap();
        assert!(geom.total_modules > 0);
        assert!(!geom.bars.is_empty());
    }

    #[test]
    fn test_code128_hello_geometry() {
        let geom = encode_code128(&[Code128Symbol::Data("HELLO".to_string())]).unwrap();
        // All-alphabetic, so Code B is already minimal:
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
    fn test_code128_only_start_symbol() {
        let geom = encode_code128(&[Code128Symbol::StartB]).unwrap();
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

    // ---- Code C packing: the defect this planner exists to fix ----

    #[test]
    fn packs_digit_runs_into_code_c() {
        // Start C + 10 pairs + checksum + stop, not Start B + 20 chars.
        assert_eq!(modules("12345678901234567890"), 10 * 11 + 35);
    }

    #[test]
    fn packs_digit_run_after_an_alpha_prefix() {
        // p1's garment vin: 'G' in Code B, latch to C, then three pairs.
        assert_eq!(modules("G262134"), 5 * 11 + 35);
    }

    #[test]
    fn packs_digits_across_data_segment_boundaries() {
        // GS1 emits one Data per AI field with no separator after a fixed-length
        // AI, so these two segments concatenate into a 24-digit run. Planning
        // per segment would leave the boundary pair unpacked.
        let split = encode_code128(&[
            Code128Symbol::FNC1,
            Code128Symbol::Data("0112345678901234".to_string()),
            Code128Symbol::Data("17260101".to_string()),
        ])
        .unwrap();
        assert_eq!(split.total_modules, modules("\x1d011234567890123417260101"));
        assert_eq!(split.total_modules, 13 * 11 + 35);
    }

    #[test]
    fn short_digit_runs_do_not_pay_for_a_latch() {
        // Latching to Code C costs a symbol, so it only pays from four digits.
        assert_eq!(modules("A12"), 3 * 11 + 35);
        assert_eq!(modules("A123"), 4 * 11 + 35);
        assert_eq!(modules("A1234"), 4 * 11 + 35); // latch + 2 pairs beats 4 chars
    }

    #[test]
    fn odd_digit_runs_latch_back_out() {
        // Trailing odd digit cannot be a Code C pair.
        assert_eq!(modules("12345"), 4 * 11 + 35);
        assert_eq!(modules("1234567"), 5 * 11 + 35);
    }

    // ---- Oracle parity ----

    /// Expected symbol counts from sklib's `min_symbol_count` — an independent
    /// exact minimal encoder (`sklib/barcodes/code128.py`). `\x1d` marks FNC1.
    /// Corpus is printable ASCII + FNC1, where the latch-only planner is
    /// provably optimal; see `cost_table` on the SHIFT omission.
    const ORACLE: &[(&str, u32)] = &[
        ("ABC123", 6),
        ("009312345678901234", 9),
        ("12345678901234567890", 10),
        ("SK-000123", 7),
        ("G262134", 5),
        ("G1234567", 6),
        ("GRM123456", 7),
        ("HELLO", 5),
        ("hello", 5),
        ("1", 1),
        ("12", 1),
        ("123", 3),
        ("1234", 2),
        ("12345", 4),
        ("123456", 3),
        ("1234567", 5),
        ("12345678", 4),
        ("A1", 2),
        ("A12", 3),
        ("A123", 4),
        ("A1234", 4),
        ("A12345", 5),
        ("A123456", 5),
        ("A1234567", 6),
        ("1A", 2),
        ("12A", 3),
        ("123A", 4),
        ("1234A", 4),
        ("12345A", 5),
        ("123456A", 5),
        ("1234567A", 6),
        ("A1234B", 6),
        ("A12345B", 7),
        ("AB1234CD", 8),
        ("AB12345CD", 9),
        ("12345678901234567890123456789012345678901234", 22),
        ("abc12345678901234xyz", 15),
        ("Item-42-Qty-00001234", 17),
        ("\x1d0120197344223371", 9),
        ("\x1d01201973442233712100000001", 14),
        ("\x1d011234567890123417260101", 13),
        ("\x1d0112345678901234", 9),
        ("\x1d10BATCH\x1d0112345678901234", 18),
        ("\x1d011234567890123410BATCH123", 19),
        ("\x1d00312345678901234567", 11),
        ("\x1d01123456789012341726010110LOT-7", 20),
        ("\x1d123\x1d456", 8),
        ("\x1d12345\x1d6", 7),
        ("Y3Kf67oJMJ0y", 12),
        ("57Y", 3),
        ("JCEP255lUT7249529BbHW5T fg", 25),
        ("5y7o8LR473OH4R8BV37iwP", 22),
        ("f164f219", 8),
        ("94wVK97jW38", 11),
        ("hG9-2k293", 9),
        ("64CWu4xHZz 79BN", 15),
        ("G4u6", 4),
        ("UyWH1yt12HypR6dm8", 17),
        ("T9KJ2Z7I6", 9),
        ("Y7MW5NUAZ", 9),
        ("7fABiUvdIt2qT77u6", 17),
        ("NAxF", 4),
        ("lQh54T2812B", 11),
        ("bZ3eF2Fj74Mqt7527Kn2IHC9Fq", 26),
        ("6U8698I3/9Tc788", 15),
        ("9B70a7702Y0Nt", 13),
        ("91H0c518It291R", 14),
        (".LlLb7T1613/0h343 2", 19),
        ("Wfd41D", 6),
        ("0YQF98N5S3j.4/4140O2j", 21),
        ("57eF22CFp223.4N2Jm6", 19),
        ("6X5xFbo4k4253S4HFrF8W8A", 23),
        ("1.j064U55tq 65AC1IP0x678", 24),
        ("62Ea33GWqF3qKTS6V4rL2LbG3", 25),
        ("nns56W9NKM6", 11),
        ("0SV7T15DQeC95s4C73T9JpJRc4", 26),
        ("y4B2K32i Y-55Jf069OYoZM2c", 25),
        ("5O", 2),
        ("F5W36J40R8I490YOi6S3w669R5O8", 28),
        ("KOLB34", 6),
        ("VKU3", 4),
        ("5I6i38xY614YX", 13),
        ("53N2B84m2MN6SOH", 15),
        ("8yP82eZ5pZu2", 12),
        ("N0qeA207441EdBn/", 15),
        ("KV", 2),
        ("ge", 2),
        ("3j36446511Z", 9),
        ("T451OQq86617r77E0U", 18),
        ("0Q61cmr5o23j49H226R173", 22),
        ("XLc91UJ99PxU9DT 9bYgF.WtA", 25),
        ("0999Q9GL3PLMH1INDR0EFy0R3", 24),
        ("WnxJL7", 6),
        ("k9P2EQ652 7V", 12),
        ("T17g6k06n9f", 11),
        ("L16ZL82Dc1-lS1xCu56 ", 20),
        ("e8U0", 4),
        ("HX68cAOEWNn-3I5C", 16),
        ("c8k", 3),
        ("Z8AfIHUWnc7cd4H.68U8SEqP1SvQ", 28),
        ("O53uUV38WyOUYY", 14),
        ("2MV6SmQ6wTHy2", 13),
        ("V8G", 3),
        ("KT44s9NFwQd9v", 13),
        (".2", 2),
        ("23.X1qALd09W", 12),
        ("2WI2UelLGzDl4", 13),
        ("Np6AZ5140GwRA5MEzNST", 20),
    ];

    #[test]
    fn matches_the_minimal_encoding_oracle() {
        for (payload, count) in ORACLE {
            // Start + data + checksum symbols are 11 modules each; stop is 13.
            let expected = count * 11 + 35;
            assert_eq!(
                modules(payload),
                expected,
                "payload {:?} should encode in {} symbols",
                payload,
                count
            );
        }
    }

    #[test]
    fn shift_divergence_is_bounded_to_one_symbol() {
        // The one documented gap from the oracle: barcoders cannot express
        // SHIFT, so alternating Code-A-only and Code-B-only characters latch
        // back and forth. The oracle plans this in 6 symbols; we take 7.
        // Nothing that reaches a label is in this class.
        assert_eq!(modules("\u{1}a\u{2}b"), 7 * 11 + 35);
    }

    // ---- Backend round-trip and input validation ----

    #[test]
    fn every_latch_direction_round_trips_through_barcoders() {
        // Cheap insurance that barcoders accepts each in-band switch marker
        // from each code set — the planner emits all six directions.
        for data in [
            "\u{C0}\u{1}\u{181}a",    // A -> B
            "\u{C0}\u{1}\u{106}1234", // A -> C
            "\u{181}a\u{C0}\u{1}",    // B -> A
            "\u{181}a\u{106}1234",    // B -> C
            "\u{106}1234\u{C0}\u{1}", // C -> A
            "\u{106}1234\u{181}a",    // C -> B
        ] {
            assert!(
                Code128::new(data).is_ok(),
                "barcoders rejected latch sequence {:?}",
                data
            );
        }
    }

    #[test]
    fn encodes_every_function_character() {
        // FNC2-4 exist in code sets A and B only, so they also force a digit
        // run out of Code C.
        for fnc in [
            Code128Symbol::FNC1,
            Code128Symbol::FNC2,
            Code128Symbol::FNC3,
            Code128Symbol::FNC4,
        ] {
            let geom = encode_code128(&[
                Code128Symbol::Data("1234".to_string()),
                fnc.clone(),
                Code128Symbol::Data("5678".to_string()),
            ])
            .unwrap();
            assert!(geom.total_modules > 0, "{:?} failed to encode", fnc);
        }
    }

    #[test]
    fn rejects_non_ascii_data() {
        let err = encode_code128(&[Code128Symbol::Data("café".to_string())]).unwrap_err();
        assert!(matches!(err, RubarError::InvalidCharacter { .. }));
    }

    #[test]
    fn rejects_switch_markers_smuggled_in_as_data() {
        // barcoders takes code-set switches in-band, so an unvalidated 'Ć' in a
        // Data payload would silently latch to Code C instead of encoding.
        for smuggled in ["A\u{106}12", "A\u{179}B", "A\u{C0}B"] {
            assert!(
                encode_code128(&[Code128Symbol::Data(smuggled.to_string())]).is_err(),
                "should reject in-band marker in {:?}",
                smuggled
            );
        }
    }

    #[test]
    fn encodes_the_full_ascii_range() {
        // Every ASCII character must encode, including DEL (127), which
        // barcoders spells as a different character in its Code B column.
        for c in 0u8..=127 {
            let data = String::from_utf8(vec![b'A', c, b'B']).unwrap();
            assert!(
                encode_code128(&[Code128Symbol::Data(data)]).is_ok(),
                "failed to encode ASCII {}",
                c
            );
        }
    }

    #[test]
    fn pinned_start_set_is_honoured() {
        // Pinning Code A costs one latch here; without the pin the planner
        // would start in Code C directly.
        assert_eq!(modules("123456"), 3 * 11 + 35);
        let pinned = encode_code128(&[
            Code128Symbol::StartA,
            Code128Symbol::Data("123456".to_string()),
        ])
        .unwrap();
        assert_eq!(pinned.total_modules, 4 * 11 + 35);
    }

    #[test]
    fn pinned_start_c_tolerates_an_odd_digit_count() {
        // Previously an error: Code C carries pairs, so a trailing lone digit
        // needs a latch back out. The planner emits one.
        let geom = encode_code128(&[
            Code128Symbol::StartC,
            Code128Symbol::Data("12345".to_string()),
        ])
        .unwrap();
        assert_eq!(geom.total_modules, 4 * 11 + 35);
    }
}
