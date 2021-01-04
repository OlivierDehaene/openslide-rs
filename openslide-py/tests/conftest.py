import pytest

from pathlib import Path
from openslide_py import OpenSlide


@pytest.fixture
def assets() -> Path:
    return Path(__file__).parent / "assets"


@pytest.fixture
def missing_file(assets):
    return assets / "__missing"


@pytest.fixture
def unsupported_file():
    return Path(__file__)


@pytest.fixture
def boxes_tiff(assets):
    return assets / "boxes.tiff"


@pytest.fixture
def unopenable_tiff(assets):
    return assets / "unopenable.tiff"


@pytest.fixture
def small_svs(assets):
    return assets / "small.svs"


@pytest.fixture
def unreadable_svs(assets):
    return assets / "unreadable.svs"


@pytest.fixture
def boxes_tiff_slide(boxes_tiff):
    return OpenSlide(boxes_tiff)
