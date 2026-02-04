import pytest
from rubar import Ean8, RubarError


def test_ean8_7_digits():
    bc = Ean8("1234567")
    geom = bc.geometry()
    assert geom.total_modules == 67


def test_ean8_8_digits_valid_checksum():
    # 12345670 has valid checksum
    bc = Ean8("12345670")
    geom = bc.geometry()
    assert geom.total_modules == 67


def test_ean8_svg():
    bc = Ean8("1234567")
    svg = bc.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg


def test_ean8_png():
    bc = Ean8("1234567")
    png = bc.render_png(400, 100, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_ean8_invalid_checksum():
    with pytest.raises(RubarError):
        Ean8("12345671")


def test_ean8_invalid_length():
    with pytest.raises(RubarError):
        Ean8("12345")


def test_ean8_non_digit():
    with pytest.raises(RubarError):
        Ean8("123456A")
