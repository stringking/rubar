//! Python bindings for rubar-core.

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};

use rubar_core::{
    encode_code128, encode_code39, encode_datamatrix, encode_ean8, encode_itf, encode_qr,
    encode_upc_a, gs1 as core_gs1, render_linear_png, render_linear_svg, render_matrix_png,
    render_matrix_svg, Bar, Code128Symbol, LinearGeometry, MatrixGeometry, RubarError, Unit,
};

// ============================================================================
// Error
// ============================================================================

pyo3::create_exception!(_rubar, PyRubarError, PyException);

/// Convert a `rubar_core::Result<T>` into a `PyResult<T>`. The orphan rule
/// prevents a blanket `From<RubarError> for PyErr`, so this trait extension
/// gives us the same ergonomics (`result.into_py_result()?`).
trait IntoPyResult<T> {
    fn into_py_result(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for rubar_core::Result<T> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e: RubarError| PyRubarError::new_err(e.to_string()))
    }
}

// ============================================================================
// Geometry wrappers
// ============================================================================

#[pyclass(frozen, skip_from_py_object, name = "Bar")]
#[derive(Debug, Clone, Copy)]
pub struct PyBar(Bar);

#[pymethods]
impl PyBar {
    #[getter]
    fn x(&self) -> u32 {
        self.0.x
    }
    #[getter]
    fn width(&self) -> u32 {
        self.0.width
    }
    fn __repr__(&self) -> String {
        format!("Bar(x={}, width={})", self.0.x, self.0.width)
    }
}

#[pyclass(frozen, skip_from_py_object, name = "LinearGeometry")]
#[derive(Debug, Clone)]
pub struct PyLinearGeometry(LinearGeometry);

#[pymethods]
impl PyLinearGeometry {
    #[getter]
    fn bars(&self) -> Vec<PyBar> {
        self.0.bars.iter().copied().map(PyBar).collect()
    }
    #[getter]
    fn total_modules(&self) -> u32 {
        self.0.total_modules
    }
    fn __repr__(&self) -> String {
        format!(
            "LinearGeometry(bars=[...{} bars], total_modules={})",
            self.0.bars.len(),
            self.0.total_modules
        )
    }
}

#[pyclass(frozen, skip_from_py_object, name = "MatrixGeometry")]
#[derive(Debug, Clone)]
pub struct PyMatrixGeometry(MatrixGeometry);

#[pymethods]
impl PyMatrixGeometry {
    #[getter]
    fn modules(&self) -> Vec<Vec<bool>> {
        self.0.modules.clone()
    }
    #[getter]
    fn width(&self) -> u32 {
        self.0.width
    }
    #[getter]
    fn height(&self) -> u32 {
        self.0.height
    }
    fn is_square(&self) -> bool {
        self.0.is_square()
    }
    fn __repr__(&self) -> String {
        format!(
            "MatrixGeometry(width={}, height={})",
            self.0.width, self.0.height
        )
    }
}

// ============================================================================
// Code 128 symbol markers (Python-visible)
// ============================================================================

#[pyclass(frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct Data {
    #[pyo3(get)]
    pub value: String,
}

#[pymethods]
impl Data {
    #[new]
    pub fn new(value: String) -> Self {
        Data { value }
    }
    fn __repr__(&self) -> String {
        format!("Data(\"{}\")", self.value)
    }
}

macro_rules! marker_symbol {
    ($name:ident) => {
        #[pyclass(frozen, skip_from_py_object)]
        #[derive(Debug, Clone)]
        pub struct $name;

        #[pymethods]
        impl $name {
            #[new]
            pub fn new() -> Self {
                $name
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

marker_symbol!(FNC1);
marker_symbol!(FNC2);
marker_symbol!(FNC3);
marker_symbol!(FNC4);
marker_symbol!(StartA);
marker_symbol!(StartB);
marker_symbol!(StartC);

fn extract_symbols(pylist: Vec<Bound<'_, PyAny>>) -> PyResult<Vec<Code128Symbol>> {
    let mut out = Vec::with_capacity(pylist.len());
    for sym in pylist {
        if let Ok(data) = sym.extract::<Data>() {
            out.push(Code128Symbol::Data(data.value));
        } else if sym.is_instance_of::<FNC1>() {
            out.push(Code128Symbol::FNC1);
        } else if sym.is_instance_of::<FNC2>() {
            out.push(Code128Symbol::FNC2);
        } else if sym.is_instance_of::<FNC3>() {
            out.push(Code128Symbol::FNC3);
        } else if sym.is_instance_of::<FNC4>() {
            out.push(Code128Symbol::FNC4);
        } else if sym.is_instance_of::<StartA>() {
            out.push(Code128Symbol::StartA);
        } else if sym.is_instance_of::<StartB>() {
            out.push(Code128Symbol::StartB);
        } else if sym.is_instance_of::<StartC>() {
            out.push(Code128Symbol::StartC);
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected Data, FNC1, FNC2, FNC3, FNC4, StartA, StartB, or StartC",
            ));
        }
    }
    Ok(out)
}

// Shared helpers for linear barcode classes — these are called from each
// class's #[pymethods] block (PyO3 disallows `macro_rules!` inside pymethods).

fn linear_geometry(geom: &LinearGeometry) -> PyLinearGeometry {
    PyLinearGeometry(geom.clone())
}

fn linear_render_svg(geom: &LinearGeometry, quiet_zone_modules: u32) -> String {
    render_linear_svg(geom, quiet_zone_modules)
}

fn linear_render_png<'py>(
    py: Python<'py>,
    geom: &LinearGeometry,
    width: f64,
    height: f64,
    unit: &str,
    dpi: Option<u32>,
    quiet_zone_modules: u32,
) -> PyResult<Bound<'py, PyBytes>> {
    let unit = Unit::from_str(unit).into_py_result()?;
    let data =
        render_linear_png(geom, width, height, unit, dpi, quiet_zone_modules).into_py_result()?;
    Ok(PyBytes::new(py, &data))
}

// ============================================================================
// Linear barcode classes
// ============================================================================

#[pyclass(frozen, skip_from_py_object)]
pub struct Code128 {
    geometry: LinearGeometry,
}

#[pymethods]
impl Code128 {
    #[new]
    fn new(symbols: Vec<Bound<'_, PyAny>>) -> PyResult<Self> {
        let rust_symbols = extract_symbols(symbols)?;
        let geometry = encode_code128(&rust_symbols).into_py_result()?;
        Ok(Code128 { geometry })
    }

    fn geometry(&self) -> PyLinearGeometry {
        linear_geometry(&self.geometry)
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        linear_render_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        linear_render_png(py, &self.geometry, width, height, unit, dpi, quiet_zone_modules)
    }
}

#[pyclass(frozen, skip_from_py_object)]
pub struct Code39 {
    geometry: LinearGeometry,
}

#[pymethods]
impl Code39 {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_code39(data).into_py_result()?;
        Ok(Code39 { geometry })
    }

    fn geometry(&self) -> PyLinearGeometry {
        linear_geometry(&self.geometry)
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        linear_render_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        linear_render_png(py, &self.geometry, width, height, unit, dpi, quiet_zone_modules)
    }
}

#[pyclass(frozen, skip_from_py_object)]
pub struct UpcA {
    geometry: LinearGeometry,
}

#[pymethods]
impl UpcA {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_upc_a(data).into_py_result()?;
        Ok(UpcA { geometry })
    }

    fn geometry(&self) -> PyLinearGeometry {
        linear_geometry(&self.geometry)
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        linear_render_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        linear_render_png(py, &self.geometry, width, height, unit, dpi, quiet_zone_modules)
    }
}

#[pyclass(frozen, skip_from_py_object)]
pub struct Ean8 {
    geometry: LinearGeometry,
}

#[pymethods]
impl Ean8 {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_ean8(data).into_py_result()?;
        Ok(Ean8 { geometry })
    }

    fn geometry(&self) -> PyLinearGeometry {
        linear_geometry(&self.geometry)
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        linear_render_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        linear_render_png(py, &self.geometry, width, height, unit, dpi, quiet_zone_modules)
    }
}

#[pyclass(frozen, skip_from_py_object)]
pub struct Itf {
    geometry: LinearGeometry,
}

#[pymethods]
impl Itf {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_itf(data).into_py_result()?;
        Ok(Itf { geometry })
    }

    fn geometry(&self) -> PyLinearGeometry {
        linear_geometry(&self.geometry)
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        linear_render_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        linear_render_png(py, &self.geometry, width, height, unit, dpi, quiet_zone_modules)
    }
}

// ============================================================================
// QR Code
// ============================================================================

#[pyclass(frozen, skip_from_py_object)]
pub struct QrCode {
    geometry: MatrixGeometry,
}

#[pymethods]
impl QrCode {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_qr(data).into_py_result()?;
        Ok(QrCode { geometry })
    }

    fn geometry(&self) -> PyMatrixGeometry {
        PyMatrixGeometry(self.geometry.clone())
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_matrix_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let unit = Unit::from_str(unit).into_py_result()?;
        let data =
            render_matrix_png(&self.geometry, width, height, unit, dpi, quiet_zone_modules)
                .into_py_result()?;
        Ok(PyBytes::new(py, &data))
    }
}

// ============================================================================
// Data Matrix
// ============================================================================

#[pyclass(frozen, skip_from_py_object)]
pub struct DataMatrix {
    geometry: MatrixGeometry,
}

#[pymethods]
impl DataMatrix {
    /// Encode a plain Data Matrix (ECC 200).
    ///
    /// Accepts either a string (UTF-8 bytes) or a `bytes` object. Per the
    /// Data Matrix spec, prefer printable ASCII for broadest scanner support.
    #[new]
    fn new(data: Bound<'_, PyAny>) -> PyResult<Self> {
        let bytes = extract_bytes_or_str(&data)?;
        let geometry = encode_datamatrix(&bytes, false).into_py_result()?;
        Ok(DataMatrix { geometry })
    }

    /// Encode a GS1 Data Matrix from the canonical parenthesized AI form,
    /// e.g. `(01)12345678901234(10)BATCH123`.
    ///
    /// Fixed-length AIs are validated; variable-length AIs get a `\x1D`
    /// (GS) separator before the next field automatically, and the FNC1
    /// designator codeword is inserted at the start of the symbol.
    #[classmethod]
    fn gs1(_cls: &Bound<'_, PyType>, value: &str) -> PyResult<Self> {
        let fields = core_gs1::parse(value).into_py_result()?;
        let payload = core_gs1::to_datamatrix_bytes(&fields);
        let geometry = encode_datamatrix(&payload, true).into_py_result()?;
        Ok(DataMatrix { geometry })
    }

    fn geometry(&self) -> PyMatrixGeometry {
        PyMatrixGeometry(self.geometry.clone())
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_matrix_svg(&self.geometry, quiet_zone_modules)
    }

    #[pyo3(signature = (width, height, *, unit = "in", dpi = None, quiet_zone_modules = 0))]
    fn render_png<'py>(
        &self,
        py: Python<'py>,
        width: f64,
        height: f64,
        unit: &str,
        dpi: Option<u32>,
        quiet_zone_modules: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let unit = Unit::from_str(unit).into_py_result()?;
        let data =
            render_matrix_png(&self.geometry, width, height, unit, dpi, quiet_zone_modules)
                .into_py_result()?;
        Ok(PyBytes::new(py, &data))
    }
}

fn extract_bytes_or_str(data: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(s) = data.extract::<&str>() {
        return Ok(s.as_bytes().to_vec());
    }
    if let Ok(b) = data.cast::<PyBytes>() {
        return Ok(b.as_bytes().to_vec());
    }
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "expected str or bytes",
    ))
}

// ============================================================================
// PyO3 Module
// ============================================================================

#[pymodule]
fn _rubar(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Barcode classes
    m.add_class::<Code128>()?;
    m.add_class::<Code39>()?;
    m.add_class::<UpcA>()?;
    m.add_class::<Ean8>()?;
    m.add_class::<Itf>()?;
    m.add_class::<QrCode>()?;
    m.add_class::<DataMatrix>()?;

    // Geometry classes
    m.add_class::<PyLinearGeometry>()?;
    m.add_class::<PyMatrixGeometry>()?;
    m.add_class::<PyBar>()?;

    // Code 128 symbols
    m.add_class::<Data>()?;
    m.add_class::<FNC1>()?;
    m.add_class::<FNC2>()?;
    m.add_class::<FNC3>()?;
    m.add_class::<FNC4>()?;
    m.add_class::<StartA>()?;
    m.add_class::<StartB>()?;
    m.add_class::<StartC>()?;

    // Error type
    m.add("RubarError", py.get_type::<PyRubarError>())?;

    Ok(())
}
