use pyo3::prelude::*;

/// A single dark bar in a linear barcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[pyclass(frozen)]
pub struct Bar {
    #[pyo3(get)]
    pub x: u32,
    #[pyo3(get)]
    pub width: u32,
}

#[pymethods]
impl Bar {
    fn __repr__(&self) -> String {
        format!("Bar(x={}, width={})", self.x, self.width)
    }
}

/// Module-based geometry for linear barcodes
#[derive(Debug, Clone)]
#[pyclass(frozen)]
pub struct LinearGeometry {
    #[pyo3(get)]
    pub bars: Vec<Bar>,
    #[pyo3(get)]
    pub total_modules: u32,
}

#[pymethods]
impl LinearGeometry {
    fn __repr__(&self) -> String {
        format!(
            "LinearGeometry(bars=[...{} bars], total_modules={})",
            self.bars.len(),
            self.total_modules
        )
    }
}

/// Module-based geometry for matrix barcodes
#[derive(Debug, Clone)]
#[pyclass(frozen)]
pub struct MatrixGeometry {
    #[pyo3(get)]
    pub modules: Vec<Vec<bool>>,
    #[pyo3(get)]
    pub size: u32,
}

#[pymethods]
impl MatrixGeometry {
    fn __repr__(&self) -> String {
        format!("MatrixGeometry(size={})", self.size)
    }
}
