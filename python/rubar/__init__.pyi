from typing import List, Optional, Union

class Bar:
    @property
    def x(self) -> int: ...
    @property
    def width(self) -> int: ...

class LinearGeometry:
    @property
    def bars(self) -> List[Bar]: ...
    @property
    def total_modules(self) -> int: ...

class MatrixGeometry:
    @property
    def modules(self) -> List[List[bool]]: ...
    @property
    def width(self) -> int: ...
    @property
    def height(self) -> int: ...
    def is_square(self) -> bool: ...

class Data:
    def __init__(self, value: str) -> None: ...
    @property
    def value(self) -> str: ...

class FNC1:
    def __init__(self) -> None: ...

class FNC2:
    def __init__(self) -> None: ...

class FNC3:
    def __init__(self) -> None: ...

class FNC4:
    def __init__(self) -> None: ...

class StartA:
    def __init__(self) -> None: ...

class StartB:
    def __init__(self) -> None: ...

class StartC:
    def __init__(self) -> None: ...

class Code128:
    def __init__(self, symbols: List[Data | FNC1 | FNC2 | FNC3 | FNC4 | StartA | StartB | StartC]) -> None: ...
    def geometry(self) -> LinearGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class Code39:
    def __init__(self, data: str) -> None: ...
    def geometry(self) -> LinearGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class UpcA:
    def __init__(self, data: str) -> None: ...
    def geometry(self) -> LinearGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class Ean8:
    def __init__(self, data: str) -> None: ...
    def geometry(self) -> LinearGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class Itf:
    def __init__(self, data: str) -> None: ...
    def geometry(self) -> LinearGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class QrCode:
    def __init__(self, data: str) -> None: ...
    def geometry(self) -> MatrixGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class DataMatrix:
    """Data Matrix (ECC 200) barcode."""

    def __init__(self, data: Union[str, bytes]) -> None: ...
    @classmethod
    def gs1(cls, value: str) -> "DataMatrix":
        """Encode a GS1 Data Matrix from the parenthesized AI form,
        e.g. ``(01)12345678901234(10)BATCH123``."""
    def geometry(self) -> MatrixGeometry: ...
    def render_svg(self, *, quiet_zone_modules: int = 0) -> str: ...
    def render_png(
        self,
        width: float,
        height: float,
        *,
        unit: str = "in",
        dpi: Optional[int] = None,
        quiet_zone_modules: int = 0,
    ) -> bytes: ...

class RubarError(Exception): ...
