import pytest
from rubar import Code39, RubarError


def test_code39_basic():
    bc = Code39("HELLO")
    geom = bc.geometry()
    assert geom.total_modules > 0
    assert len(geom.bars) > 0


def test_code39_with_numbers():
    bc = Code39("ABC123")
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code39_special_chars():
    bc = Code39("A-B.C")
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code39_lowercase_converted():
    bc = Code39("hello")
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code39_svg():
    bc = Code39("TEST")
    svg = bc.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg


def test_code39_png():
    bc = Code39("TEST")
    png = bc.render_png(400, 100, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_code39_invalid_char():
    with pytest.raises(RubarError):
        Code39("ABC@123")
