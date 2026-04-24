import pytest
from rubar import DataMatrix, RubarError


def test_datamatrix_basic_str():
    dm = DataMatrix("HELLO")
    geom = dm.geometry()
    # Smallest ECC200 square that fits "HELLO" is 10x10
    assert geom.width >= 10
    assert geom.is_square()
    assert len(geom.modules) == geom.height
    assert len(geom.modules[0]) == geom.width


def test_datamatrix_basic_bytes():
    dm = DataMatrix(b"HELLO")
    geom = dm.geometry()
    assert geom.width >= 10


def test_datamatrix_rejects_other_types():
    with pytest.raises(TypeError):
        DataMatrix(12345)


def test_datamatrix_gs1():
    # GS1 DataMatrix from parenthesized AI form
    dm = DataMatrix.gs1("(01)12345678901234(17)260101(10)BATCH123")
    geom = dm.geometry()
    assert geom.width >= 10


def test_datamatrix_gs1_mixed_fixed_and_variable():
    dm = DataMatrix.gs1("(10)BATCH(01)12345678901234")
    geom = dm.geometry()
    assert geom.width >= 10


def test_datamatrix_gs1_rejects_malformed():
    with pytest.raises(RubarError):
        DataMatrix.gs1("01)12345678901234")


def test_datamatrix_gs1_rejects_wrong_fixed_length():
    with pytest.raises(RubarError):
        DataMatrix.gs1("(01)123")


def test_datamatrix_svg():
    dm = DataMatrix("TEST")
    svg = dm.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg


def test_datamatrix_svg_quiet_zone():
    dm = DataMatrix("TEST")
    svg = dm.render_svg(quiet_zone_modules=2)
    assert "viewBox" in svg


def test_datamatrix_png_pixels():
    dm = DataMatrix("TEST")
    png = dm.render_png(200, 200, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_datamatrix_png_inches():
    dm = DataMatrix("TEST")
    png = dm.render_png(1.0, 1.0, unit="in", dpi=300)
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_datamatrix_shape_square():
    # Payload that would otherwise pick rectangular — force square
    dm = DataMatrix.gs1("(10)BATCH(01)12345678901234", shape="square")
    geom = dm.geometry()
    assert geom.is_square()


def test_datamatrix_shape_rectangular():
    # Small payload that normally picks square — force rectangular
    dm = DataMatrix("HI", shape="rectangular")
    geom = dm.geometry()
    assert not geom.is_square()


def test_datamatrix_shape_any_default():
    # Default shape allows the encoder to pick optimally
    dm = DataMatrix.gs1("(10)BATCH(01)12345678901234")
    geom = dm.geometry()
    # No assertion on shape — just that it encodes successfully
    assert geom.width >= 8


def test_datamatrix_invalid_shape():
    with pytest.raises(ValueError):
        DataMatrix("HELLO", shape="triangular")
