use pyo3::prelude::*;

/// Symbol elements for Code 128 sequences
#[derive(Debug, Clone)]
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

// Python wrapper classes for Code 128 symbols

#[pyclass(frozen)]
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

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct FNC1;

#[pymethods]
impl FNC1 {
    #[new]
    pub fn new() -> Self {
        FNC1
    }

    fn __repr__(&self) -> String {
        "FNC1".to_string()
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct FNC2;

#[pymethods]
impl FNC2 {
    #[new]
    pub fn new() -> Self {
        FNC2
    }

    fn __repr__(&self) -> String {
        "FNC2".to_string()
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct FNC3;

#[pymethods]
impl FNC3 {
    #[new]
    pub fn new() -> Self {
        FNC3
    }

    fn __repr__(&self) -> String {
        "FNC3".to_string()
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct FNC4;

#[pymethods]
impl FNC4 {
    #[new]
    pub fn new() -> Self {
        FNC4
    }

    fn __repr__(&self) -> String {
        "FNC4".to_string()
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct StartA;

#[pymethods]
impl StartA {
    #[new]
    pub fn new() -> Self {
        StartA
    }

    fn __repr__(&self) -> String {
        "StartA".to_string()
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct StartB;

#[pymethods]
impl StartB {
    #[new]
    pub fn new() -> Self {
        StartB
    }

    fn __repr__(&self) -> String {
        "StartB".to_string()
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct StartC;

#[pymethods]
impl StartC {
    #[new]
    pub fn new() -> Self {
        StartC
    }

    fn __repr__(&self) -> String {
        "StartC".to_string()
    }
}
