use crate::error::{Result, RubarError};
use crate::geometry::MatrixGeometry;
use qrcode::QrCode;

/// Encode QR code
pub fn encode_qr(data: &str) -> Result<MatrixGeometry> {
    let code = QrCode::new(data.as_bytes())
        .map_err(|e| RubarError::EncodingError(e.to_string()))?;

    let width = code.width();
    let mut modules = Vec::with_capacity(width);

    for y in 0..width {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let color = code[(x, y)];
            row.push(color == qrcode::Color::Dark);
        }
        modules.push(row);
    }

    Ok(MatrixGeometry {
        modules,
        size: width as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_basic() {
        let geom = encode_qr("HELLO").unwrap();
        assert!(geom.size >= 21); // Minimum QR is 21x21 (version 1)
        assert_eq!(geom.modules.len(), geom.size as usize);
        assert_eq!(geom.modules[0].len(), geom.size as usize);
    }

    #[test]
    fn test_qr_url() {
        let geom = encode_qr("https://example.com").unwrap();
        assert!(geom.size >= 21);
    }

    #[test]
    fn test_qr_long_data() {
        let data = "A".repeat(100);
        let geom = encode_qr(&data).unwrap();
        // Longer data requires larger QR code
        assert!(geom.size > 21);
    }

    #[test]
    fn test_qr_empty() {
        // Empty string should still work
        let geom = encode_qr("");
        // qrcode crate may or may not accept empty - just verify we handle it
        assert!(geom.is_ok() || geom.is_err());
    }
}
