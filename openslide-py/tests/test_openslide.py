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

from openslide_py import OpenSlide
from PIL import Image


def test_detect_format_missing(missing_file):
    with pytest.raises(FileNotFoundError):
        OpenSlide.detect_format(missing_file)


def test_detect_format_unsupported(unsupported_file):
    # TODO
    with pytest.raises(OSError):
        OpenSlide.detect_format(unsupported_file)


def test_detect_format(boxes_tiff):
    assert OpenSlide.detect_format(boxes_tiff) == "generic-tiff"


def test_open_missing(missing_file):
    with pytest.raises(FileNotFoundError):
        OpenSlide(missing_file)


def test_open_unsupported(unsupported_file):
    # TODO
    with pytest.raises(OSError):
        OpenSlide(unsupported_file)


def test_open_unopenable(unopenable_tiff):
    # TODO
    with pytest.raises(OSError):
        OpenSlide(unopenable_tiff)


def test_context_manager(boxes_tiff):
    slide = OpenSlide(boxes_tiff)
    with  slide:
        assert slide.level_count == 4

    with pytest.raises(AttributeError):
        slide.level_count


def test_repr(boxes_tiff):
    slide = OpenSlide(boxes_tiff)
    assert repr(slide) == f"OpenSlide({boxes_tiff})"


def test_basic_metadata(boxes_tiff):
    slide = OpenSlide(boxes_tiff)

    assert slide.level_count == 4
    assert slide.level_dimensions == [(300, 250), (150, 125), (75, 62), (37, 31)]
    assert slide.dimensions == (300, 250)
    assert len(slide.level_downsamples) == slide.level_count
    assert slide.level_downsamples[0:2] == [1, 2]
    # TODO
    #         self.assertAlmostEqual(self.osr.level_downsamples[2], 4, places=0)
    #         self.assertAlmostEqual(self.osr.level_downsamples[3], 8, places=0)

    assert slide.get_best_level_for_downsample(0.5) == 0
    assert slide.get_best_level_for_downsample(3) == 1
    assert slide.get_best_level_for_downsample(37) == 3


def test_properties(boxes_tiff):
    slide = OpenSlide(boxes_tiff)

    assert slide.properties["openslide.vendor"] == "generic-tiff"

    with pytest.raises(KeyError):
        slide.properties["__missing"]


def test_thumbnail(boxes_tiff):
    slide = OpenSlide(boxes_tiff)
    assert slide.get_thumbnail((100, 100)).size == (100, 83)


def test_associated_images(small_svs):
    slide = OpenSlide(small_svs)

    assert slide.associated_images["thumbnail"].size == (16, 16)
    assert len([v for v in slide.associated_images]) == len(slide.associated_images)

    with pytest.raises(KeyError):
        slide.associated_images["__missing"]


def test_read_bad_region(unreadable_svs):
    slide = OpenSlide(unreadable_svs)

    assert slide.properties["openslide.vendor"] == "aperio"

    # TODO
    with pytest.raises(OSError):
        slide.read_region((0, 0), 0, (16, 16))


def test_read_bad_associated_image(unreadable_svs):
    slide = OpenSlide(unreadable_svs)

    assert slide.properties["openslide.vendor"] == "aperio"

    # TODO
    # with pytest.raises(OSError):
    slide.associated_images["thumbnail"]
