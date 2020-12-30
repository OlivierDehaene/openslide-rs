import pytest

from pathlib import Path


@pytest.fixture
def missing_file():
    return Path("__missing")


@pytest.fixture
def unsupported_file():
    return Path("conftest.py")


@pytest.fixture
def boxes_tiff():
    return Path("assets/boxes.tiff")


@pytest.fixture
def unopenable_tiff():
    return Path("assets/unopenable.tiff")


@pytest.fixture
def small_svs():
    return Path("assets/small.svs")


@pytest.fixture
def unreadable_svs():
    return Path("assets/unreadable.svs")
