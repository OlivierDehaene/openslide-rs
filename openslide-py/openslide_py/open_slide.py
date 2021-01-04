#
# openslide-python - Python bindings for the OpenSlide library
#
# Copyright (c) 2010-2014 Carnegie Mellon University
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
from PIL import Image
from typing import Union, List, Callable, Any, Tuple
from pathlib import Path
from collections import Mapping

from . import PROPERTY_NAME_BACKGROUND_COLOR
from .openslide_py import _OpenSlide, OpenSlideError


class OpenSlideMap(Mapping):
    def __init__(self, names: List[str], closure: Callable[[str], Any]):
        self.names = names
        self.closure = closure

    def keys(self):
        return self.names

    def __len__(self):
        return len(self.names)

    def __iter__(self):
        return iter(self.names)

    def __getitem__(self, key):
        if key not in self.names:
            raise KeyError(key)
        return self.closure(key)


class OpenSlide:
    """An open whole-slide image.

    close() is called automatically when the object is deleted.
    The object may be used as a context manager, in which case it will be
    closed upon exiting the context.

    If an operation fails, OpenSlideError is raised.  Note that OpenSlide
    has latching error semantics: once OpenSlideError is raised, all future
    operations on the OpenSlide object, other than close(), will fail.

    Parameters
    ----------
    filename: Union[str, Path]
    cache_size: int = 1024 * 1024 * 32
        Cache size in bytes

    Raises
    ------
    FileNotFoundError
    OpenSlideUnsupportedFormatError
    """

    def __init__(self, filename: Union[str, Path], cache_size: int = 1024 * 1024 * 32):
        if isinstance(filename, str):
            filename = Path(filename)

        self._filename = filename
        self._osr = _OpenSlide(str(filename))
        self.cache_size = cache_size

    def close(self):
        self._osr = None

    def _check_closed(self):
        if self._osr is None:
            raise OpenSlideError("Slide object was closed")

    def __enter__(self) -> "OpenSlide":
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        self.close()
        return False

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}({self._filename})"

    @classmethod
    def detect_format(cls, filename: Union[str, Path]) -> str:
        """
        Parameters
        ----------
        filename: Union[str, Path]

        Returns
        -------
        format: str
            A string describing the format vendor of the specified file.

        Raises
        ------
        OpenSlideUnsupportedFormatError
        """
        if isinstance(filename, Path):
            filename = str(filename)

        return _OpenSlide.detect_format(filename)

    @property
    def cache_size(self) -> int:
        """
        Returns
        -------
        cache_size:int
            The slide cache size
        """
        return self._cache_size

    @cache_size.setter
    def cache_size(self, size: int):
        self._check_closed()
        self._osr.set_cache_size(size)
        self._cache_size = size

    @property
    def level_count(self) -> int:
        """
        Returns
        -------
        level_count: int
            The number of levels in the image.
        """
        self._check_closed()
        return self._osr.level_count

    @property
    def level_dimensions(self) -> List[Tuple[int, int]]:
        """
        Returns
        -------
        level_dimensions: List[Tuple[int, int]]
            A list of (width, height) tuples, one for each level of the image.
            level_dimensions[n] contains the dimensions of level n.
        """

        self._check_closed()
        return self._osr.all_level_dimensions

    @property
    def level_downsamples(self) -> List[int]:
        """
        Returns
        -------
        level_downsamples: List[int]
            A list of downsampling factors for each level of the image.
            level_downsample[n] contains the downsample factor of level n.
        """
        self._check_closed()
        return self._osr.all_level_downsample

    @property
    def dimensions(self) -> Tuple[int, int]:
        """
        Returns
        -------
        dimensions: Tuple[int, int]
            A (width, height) tuple for level 0 of the image.
        """
        return self.level_dimensions[0]

    @property
    def properties(self) -> OpenSlideMap:
        """Metadata about the image.

        Returns
        -------
        properties: OpenSlideMap
            This is a map: property name -> property value.
        """
        self._check_closed()
        return OpenSlideMap(self._osr.property_names,
                            lambda name: self._osr.property(name))

    @property
    def associated_images(self) -> OpenSlideMap:
        """
        Images associated with this whole-slide image.
        Unlike in the C interface, the images accessible via this property
        are not premultiplied.

        Returns
        -------
        associated_images: OpenSlideMap
            This is a map: image name -> PIL.Image.
        """
        self._check_closed()
        return OpenSlideMap(self._osr.associated_image_names,
                            lambda name: Image.fromarray(self._osr.associated_image(name)))

    def get_best_level_for_downsample(self, downsample: float) -> int:
        """
        Returns
        -------
        best_level_for_downsample:int
            The best level for displaying the given downsample.
        """
        self._check_closed()
        return self._osr.best_level_for_downsample(downsample)

    def read_region(self, location: Tuple[int, int], level: int,
                    size: Tuple[int, int]) -> Image.Image:
        """
        Parameters
        ----------
        location: Tuple[int, int]
            (x, y) tuple giving the top left pixel in the level 0 reference frame.
        level: int
            The level number
        size: Tuple[int, int]
            (width, height) tuple giving the region size.

        Returns
        -------
        region: Image.Image
            A PIL.Image containing the contents of the region.
            Unlike in the C interface, the image data returned by this
            function is not premultiplied.
        """
        self._check_closed()
        arr = self._osr.read_region(location, level, size)
        return Image.fromarray(arr)

    def get_thumbnail(self, size: Tuple[int, int]) -> Image.Image:
        """
        Parameters
        ----------
        size: Tuple[int, int]
            (width, height) tuple giving the maximum size of the thumbnail.

        Returns
        -------
        thumbnail: Image.Image
            A PIL.Image containing an RGB thumbnail of the image.
        """
        self._check_closed()

        downsample = max(*[dim / thumb for dim, thumb in
                           zip(self.dimensions, size)])
        level = self.get_best_level_for_downsample(downsample)
        tile = self.read_region((0, 0), level, self.level_dimensions[level])

        # Apply on solid background
        bg_color = '#' + self.properties.get(PROPERTY_NAME_BACKGROUND_COLOR,
                                             'ffffff')
        thumb = Image.new('RGB', tile.size, bg_color)
        thumb.paste(tile, None, tile)
        thumb.thumbnail(size, Image.ANTIALIAS)
        return thumb
