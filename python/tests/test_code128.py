import pytest
from rubar import Code128, Data, FNC1, StartA, StartB, StartC, RubarError


def test_code128_basic():
    bc = Code128([Data("ABC123")])
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code128_geometry_immutable():
    bc = Code128([Data("HELLO")])
    geom = bc.geometry()
    assert len(geom.bars) > 0
    # All-alphabetic, so Code B is already minimal:
    # Start B (11) + 5 chars (11 each) + checksum (11) + stop (13) = 90 modules
    assert geom.total_modules == 90


def test_code128_gs1():
    bc = Code128([FNC1(), Data("0101234567890128")])
    svg = bc.render_svg()
    assert "<svg" in svg
    assert "viewBox" in svg
    # FNC1 + 16 digits packs into Code C: 9 symbols, not 17.
    assert bc.geometry().total_modules == 9 * 11 + 35


def test_code128_packs_digits_into_code_c():
    # 20 digits are 10 Code C symbols, not 20 Code B ones. Callers size a
    # barcode from its module count, so getting this wrong draws it too dense.
    bc = Code128([Data("12345678901234567890")])
    assert bc.geometry().total_modules == 10 * 11 + 35


def test_code128_plans_across_data_symbols():
    # A digit run split across Data entries still packs as one run.
    split = Code128([Data("0112345678901234"), Data("17260101")])
    whole = Code128([Data("011234567890123417260101")])
    assert split.geometry().total_modules == whole.geometry().total_modules


def test_code128_rejects_non_ascii():
    with pytest.raises(RubarError):
        Code128([Data("caf\u00e9")])


def test_code128_with_quiet_zone():
    bc = Code128([Data("TEST")])
    svg = bc.render_svg(quiet_zone_modules=10)
    assert "viewBox" in svg


def test_code128_explicit_start_a():
    bc = Code128([StartA(), Data("HELLO")])
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code128_explicit_start_b():
    bc = Code128([StartB(), Data("hello")])
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code128_explicit_start_c():
    bc = Code128([StartC(), Data("123456")])
    geom = bc.geometry()
    assert geom.total_modules > 0


def test_code128_start_symbol_pins_initial_code_set():
    # StartA costs one latch here; unpinned, the planner starts in Code C.
    assert Code128([Data("123456")]).geometry().total_modules == 3 * 11 + 35
    pinned = Code128([StartA(), Data("123456")])
    assert pinned.geometry().total_modules == 4 * 11 + 35


def test_render_png_pixels():
    bc = Code128([Data("TEST")])
    png = bc.render_png(400, 100, unit="px")
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_render_png_inches():
    bc = Code128([Data("TEST")])
    png = bc.render_png(2.0, 0.5, unit="in", dpi=300)
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_render_png_mm():
    bc = Code128([Data("TEST")])
    png = bc.render_png(50.0, 12.0, unit="mm", dpi=300)
    assert png[:8] == b"\x89PNG\r\n\x1a\n"


def test_dpi_required_for_inches():
    bc = Code128([Data("TEST")])
    with pytest.raises(RubarError):
        bc.render_png(2.0, 0.5, unit="in")  # Missing dpi


def test_dpi_required_for_mm():
    bc = Code128([Data("TEST")])
    with pytest.raises(RubarError):
        bc.render_png(50.0, 12.0, unit="mm")  # Missing dpi


def test_dpi_forbidden_for_pixels():
    bc = Code128([Data("TEST")])
    with pytest.raises(RubarError):
        bc.render_png(400, 100, unit="px", dpi=300)  # dpi not allowed


def test_invalid_unit():
    bc = Code128([Data("TEST")])
    with pytest.raises(RubarError):
        bc.render_png(400, 100, unit="cm")  # Invalid unit


def test_empty_symbols():
    # Empty list creates a minimal barcode with just start/checksum/stop
    # This is technically valid Code 128 (no data, just structure)
    bc = Code128([])
    geom = bc.geometry()
    # Minimal Code 128: Start B (11) + checksum (11) + stop (13) = 35 modules
    assert geom.total_modules == 35


def test_only_start_symbol():
    # Just a start symbol creates a minimal barcode
    bc = Code128([StartB()])
    geom = bc.geometry()
    assert geom.total_modules == 35
