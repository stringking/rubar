pub mod encode;
pub mod error;
pub mod geometry;
pub mod render;
pub mod symbol;
pub mod unit;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::encode::{encode_code128, encode_code39, encode_ean8, encode_itf, encode_qr, encode_upc_a};
use crate::error::PyRubarError;
use crate::geometry::{Bar, LinearGeometry, MatrixGeometry};
use crate::render::{render_linear_png, render_linear_svg, render_matrix_png, render_matrix_svg};
use crate::symbol::{Code128Symbol, Data, StartA, StartB, StartC, FNC1, FNC2, FNC3, FNC4};
use crate::unit::Unit;

// ============================================================================
// Code 128
// ============================================================================

#[pyclass(frozen)]
pub struct Code128 {
    geometry: LinearGeometry,
}

#[pymethods]
impl Code128 {
    #[new]
    fn new(symbols: Vec<Bound<'_, PyAny>>) -> PyResult<Self> {
        let mut rust_symbols = Vec::new();

        for sym in symbols {
            if let Ok(data) = sym.extract::<Data>() {
                rust_symbols.push(Code128Symbol::Data(data.value));
            } else if sym.is_instance_of::<FNC1>() {
                rust_symbols.push(Code128Symbol::FNC1);
            } else if sym.is_instance_of::<FNC2>() {
                rust_symbols.push(Code128Symbol::FNC2);
            } else if sym.is_instance_of::<FNC3>() {
                rust_symbols.push(Code128Symbol::FNC3);
            } else if sym.is_instance_of::<FNC4>() {
                rust_symbols.push(Code128Symbol::FNC4);
            } else if sym.is_instance_of::<StartA>() {
                rust_symbols.push(Code128Symbol::StartA);
            } else if sym.is_instance_of::<StartB>() {
                rust_symbols.push(Code128Symbol::StartB);
            } else if sym.is_instance_of::<StartC>() {
                rust_symbols.push(Code128Symbol::StartC);
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Expected Data, FNC1, FNC2, FNC3, FNC4, StartA, StartB, or StartC",
                ));
            }
        }

        let geometry = encode_code128(&rust_symbols)?;
        Ok(Code128 { geometry })
    }

    fn geometry(&self) -> LinearGeometry {
        self.geometry.clone()
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_linear_svg(&self.geometry, quiet_zone_modules)
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
        let unit = Unit::from_str(unit)?;
        let data = render_linear_png(
            &self.geometry,
            width,
            height,
            unit,
            dpi,
            quiet_zone_modules,
        )?;
        Ok(PyBytes::new_bound(py, &data))
    }
}

// ============================================================================
// Code 39
// ============================================================================

#[pyclass(frozen)]
pub struct Code39 {
    geometry: LinearGeometry,
}

#[pymethods]
impl Code39 {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_code39(data)?;
        Ok(Code39 { geometry })
    }

    fn geometry(&self) -> LinearGeometry {
        self.geometry.clone()
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_linear_svg(&self.geometry, quiet_zone_modules)
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
        let unit = Unit::from_str(unit)?;
        let data = render_linear_png(
            &self.geometry,
            width,
            height,
            unit,
            dpi,
            quiet_zone_modules,
        )?;
        Ok(PyBytes::new_bound(py, &data))
    }
}

// ============================================================================
// UPC-A
// ============================================================================

#[pyclass(frozen)]
pub struct UpcA {
    geometry: LinearGeometry,
}

#[pymethods]
impl UpcA {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_upc_a(data)?;
        Ok(UpcA { geometry })
    }

    fn geometry(&self) -> LinearGeometry {
        self.geometry.clone()
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_linear_svg(&self.geometry, quiet_zone_modules)
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
        let unit = Unit::from_str(unit)?;
        let data = render_linear_png(
            &self.geometry,
            width,
            height,
            unit,
            dpi,
            quiet_zone_modules,
        )?;
        Ok(PyBytes::new_bound(py, &data))
    }
}

// ============================================================================
// EAN-8
// ============================================================================

#[pyclass(frozen)]
pub struct Ean8 {
    geometry: LinearGeometry,
}

#[pymethods]
impl Ean8 {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_ean8(data)?;
        Ok(Ean8 { geometry })
    }

    fn geometry(&self) -> LinearGeometry {
        self.geometry.clone()
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_linear_svg(&self.geometry, quiet_zone_modules)
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
        let unit = Unit::from_str(unit)?;
        let data = render_linear_png(
            &self.geometry,
            width,
            height,
            unit,
            dpi,
            quiet_zone_modules,
        )?;
        Ok(PyBytes::new_bound(py, &data))
    }
}

// ============================================================================
// ITF (Interleaved 2 of 5)
// ============================================================================

#[pyclass(frozen)]
pub struct Itf {
    geometry: LinearGeometry,
}

#[pymethods]
impl Itf {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_itf(data)?;
        Ok(Itf { geometry })
    }

    fn geometry(&self) -> LinearGeometry {
        self.geometry.clone()
    }

    #[pyo3(signature = (*, quiet_zone_modules = 0))]
    fn render_svg(&self, quiet_zone_modules: u32) -> String {
        render_linear_svg(&self.geometry, quiet_zone_modules)
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
        let unit = Unit::from_str(unit)?;
        let data = render_linear_png(
            &self.geometry,
            width,
            height,
            unit,
            dpi,
            quiet_zone_modules,
        )?;
        Ok(PyBytes::new_bound(py, &data))
    }
}

// ============================================================================
// QR Code
// ============================================================================

#[pyclass(frozen)]
pub struct QrCode {
    geometry: MatrixGeometry,
}

#[pymethods]
impl QrCode {
    #[new]
    fn new(data: &str) -> PyResult<Self> {
        let geometry = encode_qr(data)?;
        Ok(QrCode { geometry })
    }

    fn geometry(&self) -> MatrixGeometry {
        self.geometry.clone()
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
        let unit = Unit::from_str(unit)?;
        let data = render_matrix_png(
            &self.geometry,
            width,
            height,
            unit,
            dpi,
            quiet_zone_modules,
        )?;
        Ok(PyBytes::new_bound(py, &data))
    }
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

    // Geometry classes
    m.add_class::<LinearGeometry>()?;
    m.add_class::<MatrixGeometry>()?;
    m.add_class::<Bar>()?;

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
    m.add("RubarError", py.get_type_bound::<PyRubarError>())?;

    Ok(())
}
