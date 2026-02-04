# Examples

This directory contains example scripts demonstrating rubar's barcode generation capabilities.

## Generating Examples

```bash
uv run python examples/generate_examples.py
```

Output files are written to `examples/output/` (gitignored).

## Examples

### Code 128

| File | Description |
|------|-------------|
| `code128_basic.*` | Basic Code 128 barcode encoding "HELLO-123" |
| `code128_basic_300dpi.png` | Same barcode rendered at 2"×0.5" @ 300 DPI |
| `code128_gs1.*` | GS1-128 with FNC1 symbol for supply chain applications |
| `code128_quiet_zone.*` | Code 128 with 10-module quiet zones on each side |

### Code 39

| File | Description |
|------|-------------|
| `code39.*` | Code 39 barcode encoding "CODE39-TEST" |

### UPC-A

| File | Description |
|------|-------------|
| `upc_a.*` | UPC-A barcode for "012345678905" (with valid check digit) |

### EAN-8

| File | Description |
|------|-------------|
| `ean8.*` | EAN-8 barcode for "12345670" (with valid check digit) |

### ITF-14

| File | Description |
|------|-------------|
| `itf14.*` | ITF-14 (Interleaved 2 of 5) barcode for shipping containers |

### QR Code

| File | Description |
|------|-------------|
| `qr_basic.*` | QR code encoding a URL |
| `qr_quiet_zone.*` | QR code with standard 4-module quiet zone |

## Rendering Notes

- **SVG files** use `viewBox` only (no width/height), so they scale to any size
- **PNG files** are rendered at exact pixel dimensions
- For crisp PNG rendering, use dimensions that are multiples of the module count
- Quiet zones are optional (default: 0) — add them based on your scanning requirements
