#!/usr/bin/env python3
"""Generate example barcodes demonstrating rubar's capabilities."""

from pathlib import Path

from rubar import Code128, Code39, Ean8, Itf, QrCode, UpcA
from rubar import Data, FNC1, StartA, StartB, StartC

OUTPUT_DIR = Path(__file__).parent / "output"


def main():
    OUTPUT_DIR.mkdir(exist_ok=True)

    # -------------------------------------------------------------------------
    # Code 128
    # -------------------------------------------------------------------------

    # Basic Code 128
    bc = Code128([Data("HELLO-123")])
    (OUTPUT_DIR / "code128_basic.svg").write_text(bc.render_svg())
    (OUTPUT_DIR / "code128_basic.png").write_bytes(bc.render_png(400, 100, unit="px"))
    (OUTPUT_DIR / "code128_basic_300dpi.png").write_bytes(
        bc.render_png(2.0, 0.5, unit="in", dpi=300)
    )
    print("Generated: code128_basic.svg, code128_basic.png, code128_basic_300dpi.png")

    # GS1-128 with FNC1 (used for supply chain applications)
    gs1 = Code128([StartC(), FNC1(), Data("01"), Data("12345678901234")])
    (OUTPUT_DIR / "code128_gs1.svg").write_text(gs1.render_svg())
    (OUTPUT_DIR / "code128_gs1.png").write_bytes(gs1.render_png(500, 100, unit="px"))
    print("Generated: code128_gs1.svg, code128_gs1.png")

    # Code 128 with quiet zones
    bc_quiet = Code128([Data("WITH-QUIET-ZONE")])
    (OUTPUT_DIR / "code128_quiet_zone.svg").write_text(
        bc_quiet.render_svg(quiet_zone_modules=10)
    )
    (OUTPUT_DIR / "code128_quiet_zone.png").write_bytes(
        bc_quiet.render_png(400, 100, unit="px", quiet_zone_modules=10)
    )
    print("Generated: code128_quiet_zone.svg, code128_quiet_zone.png")

    # -------------------------------------------------------------------------
    # Code 39
    # -------------------------------------------------------------------------

    c39 = Code39("CODE39-TEST")
    (OUTPUT_DIR / "code39.svg").write_text(c39.render_svg())
    (OUTPUT_DIR / "code39.png").write_bytes(c39.render_png(500, 100, unit="px"))
    print("Generated: code39.svg, code39.png")

    # -------------------------------------------------------------------------
    # UPC-A
    # -------------------------------------------------------------------------

    upc = UpcA("012345678905")
    (OUTPUT_DIR / "upc_a.svg").write_text(upc.render_svg())
    # Use width that's a multiple of modules (95) for crisp rendering
    (OUTPUT_DIR / "upc_a.png").write_bytes(upc.render_png(285, 100, unit="px"))
    print("Generated: upc_a.svg, upc_a.png")

    # -------------------------------------------------------------------------
    # EAN-8
    # -------------------------------------------------------------------------

    ean = Ean8("12345670")
    (OUTPUT_DIR / "ean8.svg").write_text(ean.render_svg())
    # Use width that's a multiple of modules (67) for crisp rendering
    (OUTPUT_DIR / "ean8.png").write_bytes(ean.render_png(201, 100, unit="px"))
    print("Generated: ean8.svg, ean8.png")

    # -------------------------------------------------------------------------
    # ITF-14
    # -------------------------------------------------------------------------

    itf = Itf("00012345678905")
    (OUTPUT_DIR / "itf14.svg").write_text(itf.render_svg())
    (OUTPUT_DIR / "itf14.png").write_bytes(itf.render_png(400, 80, unit="px"))
    print("Generated: itf14.svg, itf14.png")

    # -------------------------------------------------------------------------
    # QR Code
    # -------------------------------------------------------------------------

    qr = QrCode("https://github.com")
    size = qr.geometry().size
    pixel_size = size * 10  # 10 pixels per module for crisp rendering
    (OUTPUT_DIR / "qr_basic.svg").write_text(qr.render_svg())
    (OUTPUT_DIR / "qr_basic.png").write_bytes(
        qr.render_png(pixel_size, pixel_size, unit="px")
    )
    print(f"Generated: qr_basic.svg, qr_basic.png ({pixel_size}x{pixel_size}px)")

    # QR with quiet zone (standard is 4 modules)
    (OUTPUT_DIR / "qr_quiet_zone.svg").write_text(qr.render_svg(quiet_zone_modules=4))
    total_size = (size + 8) * 10  # size + 2*4 quiet zone modules
    (OUTPUT_DIR / "qr_quiet_zone.png").write_bytes(
        qr.render_png(total_size, total_size, unit="px", quiet_zone_modules=4)
    )
    print(f"Generated: qr_quiet_zone.svg, qr_quiet_zone.png")

    print(f"\nAll examples written to {OUTPUT_DIR}/")


if __name__ == "__main__":
    main()
