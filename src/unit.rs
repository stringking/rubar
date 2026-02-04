use crate::error::{Result, RubarError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Inches,
    Millimeters,
    Pixels,
}

impl Unit {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "in" => Ok(Unit::Inches),
            "mm" => Ok(Unit::Millimeters),
            "px" => Ok(Unit::Pixels),
            _ => Err(RubarError::InvalidUnit(s.to_string())),
        }
    }

    pub fn to_pixels(&self, value: f64, dpi: Option<u32>) -> Result<u32> {
        match self {
            Unit::Pixels => {
                if dpi.is_some() {
                    return Err(RubarError::DpiForbidden);
                }
                Ok(value.round() as u32)
            }
            Unit::Inches => {
                let dpi = dpi.ok_or(RubarError::DpiRequired)?;
                Ok((value * dpi as f64).round() as u32)
            }
            Unit::Millimeters => {
                let dpi = dpi.ok_or(RubarError::DpiRequired)?;
                Ok(((value / 25.4) * dpi as f64).round() as u32)
            }
        }
    }
}
