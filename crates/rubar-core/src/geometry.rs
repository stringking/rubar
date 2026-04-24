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

/// Module-based geometry for matrix barcodes (QR).
#[derive(Debug, Clone)]
pub struct MatrixGeometry {
    pub modules: Vec<Vec<bool>>,
    pub size: u32,
}
