use crate::error::{Result, RubarError};
use crate::geometry::{LinearGeometry, MatrixGeometry};
use crate::unit::Unit;
use tiny_skia::{Color, IntSize, Pixmap, Rect, Transform};

/// Render linear barcode to PNG
pub fn render_linear_png(
    geom: &LinearGeometry,
    width: f64,
    height: f64,
    unit: Unit,
    dpi: Option<u32>,
    quiet_zone_modules: u32,
) -> Result<Vec<u8>> {
    let pixel_width = unit.to_pixels(width, dpi)?;
    let pixel_height = unit.to_pixels(height, dpi)?;

    let size = IntSize::from_wh(pixel_width, pixel_height)
        .ok_or_else(|| RubarError::RenderingError("Invalid dimensions".to_string()))?;

    let mut pixmap = Pixmap::new(size.width(), size.height())
        .ok_or_else(|| RubarError::RenderingError("Failed to create pixmap".to_string()))?;

    // Fill with white
    pixmap.fill(Color::WHITE);

    let total_modules = geom.total_modules + 2 * quiet_zone_modules;
    let module_width = pixel_width as f32 / total_modules as f32;

    let mut paint = tiny_skia::Paint::default();
    paint.set_color(Color::BLACK);
    paint.anti_alias = false;

    // Use integer pixel boundaries to ensure bars are flush (no gaps)
    for bar in &geom.bars {
        let mx = bar.x + quiet_zone_modules;
        let x0 = (mx as f32 * module_width).round();
        let x1 = ((mx + bar.width) as f32 * module_width).round();

        if let Some(rect) = Rect::from_xywh(x0, 0.0, x1 - x0, pixel_height as f32) {
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    pixmap
        .encode_png()
        .map_err(|e| RubarError::RenderingError(e.to_string()))
}

/// Render matrix barcode to PNG
pub fn render_matrix_png(
    geom: &MatrixGeometry,
    width: f64,
    height: f64,
    unit: Unit,
    dpi: Option<u32>,
    quiet_zone_modules: u32,
) -> Result<Vec<u8>> {
    let pixel_width = unit.to_pixels(width, dpi)?;
    let pixel_height = unit.to_pixels(height, dpi)?;

    let size = IntSize::from_wh(pixel_width, pixel_height)
        .ok_or_else(|| RubarError::RenderingError("Invalid dimensions".to_string()))?;

    let mut pixmap = Pixmap::new(size.width(), size.height())
        .ok_or_else(|| RubarError::RenderingError("Failed to create pixmap".to_string()))?;

    // Fill with white
    pixmap.fill(Color::WHITE);

    let total_size = geom.size + 2 * quiet_zone_modules;
    let module_size_x = pixel_width as f32 / total_size as f32;
    let module_size_y = pixel_height as f32 / total_size as f32;

    let mut paint = tiny_skia::Paint::default();
    paint.set_color(Color::BLACK);
    paint.anti_alias = false;

    // Use integer pixel boundaries to ensure modules are flush (no gaps)
    for (y, row) in geom.modules.iter().enumerate() {
        for (x, &dark) in row.iter().enumerate() {
            if dark {
                let mx = x as u32 + quiet_zone_modules;
                let my = y as u32 + quiet_zone_modules;

                // Compute pixel boundaries as integers to avoid gaps
                let x0 = (mx as f32 * module_size_x).round();
                let x1 = ((mx + 1) as f32 * module_size_x).round();
                let y0 = (my as f32 * module_size_y).round();
                let y1 = ((my + 1) as f32 * module_size_y).round();

                if let Some(rect) = Rect::from_xywh(x0, y0, x1 - x0, y1 - y0) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        }
    }

    pixmap
        .encode_png()
        .map_err(|e| RubarError::RenderingError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Bar;

    #[test]
    fn test_linear_png_pixels() {
        let geom = LinearGeometry {
            bars: vec![Bar { x: 0, width: 2 }, Bar { x: 4, width: 1 }],
            total_modules: 10,
        };

        let png = render_linear_png(&geom, 100.0, 50.0, Unit::Pixels, None, 0).unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn test_linear_png_inches() {
        let geom = LinearGeometry {
            bars: vec![Bar { x: 0, width: 2 }],
            total_modules: 10,
        };

        let png = render_linear_png(&geom, 2.0, 0.5, Unit::Inches, Some(100), 0).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_linear_png_mm() {
        let geom = LinearGeometry {
            bars: vec![Bar { x: 0, width: 2 }],
            total_modules: 10,
        };

        let png = render_linear_png(&geom, 50.0, 10.0, Unit::Millimeters, Some(100), 0).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_linear_png_dpi_required() {
        let geom = LinearGeometry {
            bars: vec![],
            total_modules: 10,
        };

        let result = render_linear_png(&geom, 2.0, 0.5, Unit::Inches, None, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_linear_png_dpi_forbidden() {
        let geom = LinearGeometry {
            bars: vec![],
            total_modules: 10,
        };

        let result = render_linear_png(&geom, 100.0, 50.0, Unit::Pixels, Some(300), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_matrix_png_pixels() {
        let geom = MatrixGeometry {
            modules: vec![vec![true, false], vec![false, true]],
            size: 2,
        };

        let png = render_matrix_png(&geom, 100.0, 100.0, Unit::Pixels, None, 0).unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }
}
