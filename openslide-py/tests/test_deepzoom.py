#
# openslide-python - Python bindings for the OpenSlide library
#
# Copyright (c) 2016 Benjamin Gilbert
#
# This library is free software; you can redistribute it and/or modify it
# under the terms of version 2.1 of the GNU Lesser General Public License
# as published by the Free Software Foundation.
#
# This library is distributed in the hope that it will be useful, but
# WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
# or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public
# License for more details.
#
# You should have received a copy of the GNU Lesser General Public License
# along with this library; if not, write to the Free Software Foundation,
# Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
#
import pytest

from openslide_py.deepzoom import DeepZoomGenerator


@pytest.fixture
def boxes_tiff_dz(boxes_tiff_slide):
    return DeepZoomGenerator(boxes_tiff_slide, 254, 1, limit_bounds=False)


def test_repr(boxes_tiff_slide, boxes_tiff_dz):
    assert repr(boxes_tiff_dz) == f"DeepZoomGenerator({boxes_tiff_slide}, " \
                                  f"tile_size=254, overlap=1, limit_bounds=False)"


def test_metadata(boxes_tiff_dz):
    assert boxes_tiff_dz.level_count == 10
    assert boxes_tiff_dz.tile_count == 11
    assert boxes_tiff_dz.level_tiles == ((1, 1), (1, 1), (1, 1), (1, 1), (1, 1),
                                         (1, 1), (1, 1), (1, 1), (1, 1), (2, 1))

    assert boxes_tiff_dz.level_dimensions == ((1, 1), (2, 1), (3, 2), (5, 4), (10, 8),
                                              (19, 16), (38, 32), (75, 63), (150, 125),
                                              (300, 250))


def test_get_tile(boxes_tiff_dz):
    assert boxes_tiff_dz.get_tile(9, (1, 0)).size == (47, 250)


def test_get_tile_bad_level(boxes_tiff_dz):
    with pytest.raises(ValueError):
        boxes_tiff_dz.get_tile(-1, (0, 0))

    with pytest.raises(ValueError):
        boxes_tiff_dz.get_tile(10, (0, 0))


def test_get_tile_bad_address(boxes_tiff_dz):
    with pytest.raises(ValueError):
        boxes_tiff_dz.get_tile(0, (-1, 0))

    with pytest.raises(ValueError):
        boxes_tiff_dz.get_tile(0, (1, 0))


def test_get_tile_coordinates(boxes_tiff_dz):
    assert boxes_tiff_dz.get_tile_coordinates(9, (1, 0)) == ((253, 0), 0, (47, 250))


def test_get_tile_dimensions(boxes_tiff_dz):
    assert boxes_tiff_dz.get_tile_dimensions(9, (1,0)) == (47, 250)


def test_get_dzi(boxes_tiff_dz):
    assert 'http://schemas.microsoft.com/deepzoom/2008' in boxes_tiff_dz.get_dzi('jpeg')
