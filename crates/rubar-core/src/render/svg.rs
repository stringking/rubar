use crate::geometry::{LinearGeometry, MatrixGeometry};

/// Render linear barcode to SVG (viewBox only, no width/height)
pub fn render_linear_svg(geom: &LinearGeometry, quiet_zone_modules: u32) -> String {
    let total_width = geom.total_modules + 2 * quiet_zone_modules;
    let height = 1; // 1 module unit tall

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
        total_width, height
    );

    for bar in &geom.bars {
        let x = bar.x + quiet_zone_modules;
        svg.push_str(&format!(
            r#"<rect x="{}" y="0" width="{}" height="1"/>"#,
            x, bar.width
        ));
    }

    svg.push_str("</svg>");
    svg
}

/// Render matrix barcode to SVG (viewBox only, no width/height)
pub fn render_matrix_svg(geom: &MatrixGeometry, quiet_zone_modules: u32) -> String {
    let total_w = geom.width + 2 * quiet_zone_modules;
    let total_h = geom.height + 2 * quiet_zone_modules;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
        total_w, total_h
    );

    // White background
    svg.push_str(&format!(
        r#"<rect x="0" y="0" width="{}" height="{}" fill="white"/>"#,
        total_w, total_h
    ));

    for (y, row) in geom.modules.iter().enumerate() {
        for (x, &dark) in row.iter().enumerate() {
            if dark {
                let px = x as u32 + quiet_zone_modules;
                let py = y as u32 + quiet_zone_modules;
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="1" height="1"/>"#,
                    px, py
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Bar;

    #[test]
    fn test_linear_svg_basic() {
        let geom = LinearGeometry {
            bars: vec![
                Bar { x: 0, width: 2 },
                Bar { x: 4, width: 1 },
            ],
            total_modules: 10,
        };

        let svg = render_linear_svg(&geom, 0);
        assert!(svg.contains("viewBox=\"0 0 10 1\""));
        assert!(svg.contains(r#"<rect x="0" y="0" width="2" height="1"/>"#));
        assert!(svg.contains(r#"<rect x="4" y="0" width="1" height="1"/>"#));
    }

    #[test]
    fn test_linear_svg_with_quiet_zone() {
        let geom = LinearGeometry {
            bars: vec![Bar { x: 0, width: 2 }],
            total_modules: 10,
        };

        let svg = render_linear_svg(&geom, 5);
        assert!(svg.contains("viewBox=\"0 0 20 1\"")); // 10 + 2*5
        assert!(svg.contains(r#"<rect x="5" y="0" width="2" height="1"/>"#)); // x offset by 5
    }

    #[test]
    fn test_matrix_svg_basic() {
        let geom = MatrixGeometry {
            modules: vec![
                vec![true, false],
                vec![false, true],
            ],
            width: 2,
            height: 2,
        };

        let svg = render_matrix_svg(&geom, 0);
        assert!(svg.contains("viewBox=\"0 0 2 2\""));
        assert!(svg.contains(r#"<rect x="0" y="0" width="1" height="1"/>"#));
        assert!(svg.contains(r#"<rect x="1" y="1" width="1" height="1"/>"#));
    }

    #[test]
    fn test_matrix_svg_with_quiet_zone() {
        let geom = MatrixGeometry {
            modules: vec![vec![true]],
            width: 1,
            height: 1,
        };

        let svg = render_matrix_svg(&geom, 4);
        assert!(svg.contains("viewBox=\"0 0 9 9\"")); // 1 + 2*4
        assert!(svg.contains(r#"<rect x="4" y="4" width="1" height="1"/>"#)); // offset by 4
    }

    #[test]
    fn test_matrix_svg_rectangular() {
        // Data Matrix can be rectangular (e.g. 8x18)
        let geom = MatrixGeometry {
            modules: vec![vec![true, false, true, false]; 2],
            width: 4,
            height: 2,
        };

        let svg = render_matrix_svg(&geom, 0);
        assert!(svg.contains("viewBox=\"0 0 4 2\""));
    }
}
