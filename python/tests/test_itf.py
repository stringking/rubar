import pytest
from rubar import Itf, RubarError


def test_itf_basic():
    bc = Itf("12345678")
    geom = bc.geometry()
    assert geom.total_modules > 0
    assert len(geom.bars) > 0


def test_itf_14():
    # ITF-14 commonly used for shipping
    bc = Itf("00012345678905")
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_itf_svg():
    bc = Itf("12345678")
    svg = bc.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg


def test_itf_png():
    bc = Itf("12345678")
    png = bc.render_png(400, 100, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_itf_odd_length():
    with pytest.raises(RubarError):
        Itf("1234567")


def test_itf_non_digit():
    with pytest.raises(RubarError):
        Itf("12A456")
