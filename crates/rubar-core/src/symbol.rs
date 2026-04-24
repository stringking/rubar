/// Symbol elements for Code 128 sequences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Code128Symbol {
    Data(String),
    FNC1,
    FNC2,
    FNC3,
    FNC4,
    StartA,
    StartB,
    StartC,
}
