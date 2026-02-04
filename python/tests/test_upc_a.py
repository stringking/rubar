import pytest
from rubar import UpcA, RubarError


def test_upc_a_11_digits():
    bc = UpcA("01234567890")
    geom = bc.geometry()
    assert geom.total_modules == 95


def test_upc_a_12_digits_valid_checksum():
    # 012345678905 has valid checksum
    bc = UpcA("012345678905")
    geom = bc.geometry()
    assert geom.total_modules == 95


def test_upc_a_svg():
    bc = UpcA("01234567890")
    svg = bc.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg


def test_upc_a_png():
    bc = UpcA("01234567890")
    png = bc.render_png(400, 100, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_upc_a_invalid_checksum():
    with pytest.raises(RubarError):
        UpcA("012345678901")


def test_upc_a_invalid_length():
    with pytest.raises(RubarError):
        UpcA("12345")


def test_upc_a_non_digit():
    with pytest.raises(RubarError):
        UpcA("0123456789A")
