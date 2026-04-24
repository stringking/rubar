# rubar-core

Pure-Rust barcode encoding and rendering — the core of the [rubar](https://github.com/stringking/rubar) project with no Python dependency.

## Supported symbologies

| Symbology | Function |
|---|---|
| Code 128 | `encode_code128` |
| GS1-128 | `gs1::parse` + `gs1::to_symbols` + `encode_code128` |
| Code 39 | `encode_code39` |
| UPC-A | `encode_upc_a` |
| EAN-8 | `encode_ean8` |
| ITF (Interleaved 2 of 5) | `encode_itf` |
| QR Code | `encode_qr` |
| Data Matrix (ECC 200) | `encode_datamatrix` |
| GS1 Data Matrix | `gs1::parse` + `gs1::to_datamatrix_bytes` + `encode_datamatrix(..., true)` |

## Design

- **Exact module-level geometry.** Linear symbologies return `LinearGeometry { bars: Vec<Bar>, total_modules }`. Matrix symbologies return `MatrixGeometry { modules, width, height }`.
- **No implicit quiet zones.** Pass `quiet_zone_modules` to the render helpers if you want one.
- **GS1 API symmetry.** The same parenthesized `(AI)data` form works for both GS1-128 and GS1 Data Matrix via the `gs1` module.

## Example

```rust
use rubar_core::{gs1, encode_code128};

let fields = gs1::parse("(01)12345678901234(10)BATCH123")?;
let symbols = gs1::to_symbols(&fields);
let geometry = encode_code128(&symbols)?;

for bar in &geometry.bars {
    // draw a rectangle at x=bar.x, width=bar.width (in module units)
}
# Ok::<(), rubar_core::RubarError>(())
```

## Rendering

`render_linear_svg` / `render_matrix_svg` emit unitless SVG (viewBox only). `render_linear_png` / `render_matrix_png` emit PNG bytes at an exact pixel size.

Consumers that want to draw into another target (PDF, canvas, etc.) should read the `geometry()` output directly.

## Python bindings

The [`rubar`](https://pypi.org/project/rubar/) package on PyPI wraps this crate with PyO3. If you only need Rust, depend on `rubar-core` directly.
