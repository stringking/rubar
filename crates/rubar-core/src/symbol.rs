/// Symbol elements for Code 128 sequences.
///
/// `encode_code128` chooses code sets to minimise the symbol count, so a
/// `StartA`/`StartB`/`StartC` is rarely needed: it pins the *initial* code set
/// and planning continues from there. Only the first one is honoured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Code128Symbol {
    /// ASCII only — non-ASCII is rejected by `encode_code128`.
    Data(String),
    FNC1,
    FNC2,
    FNC3,
    FNC4,
    StartA,
    StartB,
    StartC,
}
