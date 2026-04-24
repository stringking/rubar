import pytest
from rubar import QrCode, RubarError


def test_qr_basic():
    qr = QrCode("HELLO")
    geom = qr.geometry()
    assert geom.width >= 21  # Minimum QR is 21x21 (version 1)
    assert geom.is_square()
    assert len(geom.modules) == geom.height
    assert len(geom.modules[0]) == geom.width


def test_qr_url():
    qr = QrCode("https://example.com")
    geom = qr.geometry()
    assert geom.width >= 21


def test_qr_long_data():
    qr = QrCode("A" * 100)
    geom = qr.geometry()
    # Longer data requires larger QR code
    assert geom.width > 21


def test_qr_svg():
    qr = QrCode("TEST")
    svg = qr.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg


def test_qr_svg_with_quiet_zone():
    qr = QrCode("TEST")
    svg = qr.render_svg(quiet_zone_modules=4)
    assert "viewBox" in svg


def test_qr_png_pixels():
    qr = QrCode("TEST")
    png = qr.render_png(200, 200, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_qr_png_inches():
    qr = QrCode("TEST")
    png = qr.render_png(2.0, 2.0, unit="in", dpi=300)
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_qr_dpi_required():
    qr = QrCode("TEST")
    with pytest.raises(RubarError):
        qr.render_png(2.0, 2.0, unit="in")


def test_qr_dpi_forbidden():
    qr = QrCode("TEST")
    with pytest.raises(RubarError):
        qr.render_png(200, 200, unit="px", dpi=300)
