/// A single dark bar in a linear barcode, measured in module units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    pub x: u32,
    pub width: u32,
}

/// Module-based geometry for linear barcodes.
#[derive(Debug, Clone)]
pub struct LinearGeometry {
    pub bars: Vec<Bar>,
    pub total_modules: u32,
}

/// Module-based geometry for matrix barcodes (QR, Data Matrix).
///
/// `modules` is indexed `[row][col]`, so `modules.len() == height as usize`
/// and each row has `width as usize` entries. For QR codes, `width == height`
/// always; Data Matrix can be rectangular.
#[derive(Debug, Clone)]
pub struct MatrixGeometry {
    pub modules: Vec<Vec<bool>>,
    pub width: u32,
    pub height: u32,
}

impl MatrixGeometry {
    /// True when the matrix is square (QR always, some Data Matrix sizes).
    pub fn is_square(&self) -> bool {
        self.width == self.height
    }
}
