from PIL import Image
from typing import Union
from pathlib import Path
from collections.abc import Mapping

from .openslide_py import _OpenSlide

PROPERTY_NAME_COMMENT          = u'openslide.comment'
PROPERTY_NAME_VENDOR           = u'openslide.vendor'
PROPERTY_NAME_QUICKHASH1       = u'openslide.quickhash-1'
PROPERTY_NAME_BACKGROUND_COLOR = u'openslide.background-color'
PROPERTY_NAME_OBJECTIVE_POWER  = u'openslide.objective-power'
PROPERTY_NAME_MPP_X            = u'openslide.mpp-x'
PROPERTY_NAME_MPP_Y            = u'openslide.mpp-y'
PROPERTY_NAME_BOUNDS_X         = u'openslide.bounds-x'
PROPERTY_NAME_BOUNDS_Y         = u'openslide.bounds-y'
PROPERTY_NAME_BOUNDS_WIDTH     = u'openslide.bounds-width'
PROPERTY_NAME_BOUNDS_HEIGHT    = u'openslide.bounds-height'


class _AssociatedImageMap(Mapping):
    def __init__(self, osr):
        self._osr = osr

    def __repr__(self):
        return f"{self.__class__.__name__} {dict(self)}"

    def __len__(self):
        return len(self._keys())

    def __iter__(self):
        return iter(self._keys())

    def _keys(self):
        return self._osr.associated_image_names

    def __getitem__(self, key):
        if key not in self._keys():
            raise KeyError()
        arr = self._osr.associated_image(key)
        return Image.fromarray(arr)


class OpenSlide:
    """An open whole-slide image.

    close() is called automatically when the object is deleted.
    The object may be used as a context manager, in which case it will be
    closed upon exiting the context.

    If an operation fails, OpenSlideError is raised.  Note that OpenSlide
    has latching error semantics: once OpenSlideError is raised, all future
    operations on the OpenSlide object, other than close(), will fail.
    """

    def __init__(self, filename: Union[str, Path]):
        """Open a whole-slide image."""
        if isinstance(filename, str):
            filename = Path(filename)

        self._filename = filename
        self._osr = _OpenSlide(str(filename))

    def close(self):
        del self._osr

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
        return False

    def __repr__(self):
        return f"{self.__class__.__name__}({self._filename})"

    @classmethod
    def detect_format(cls, filename: str):
        """Return a string describing the format vendor of the specified file.

        If the file format is not recognized, return None."""
        return _OpenSlide.detect_format(filename)

    @property
    def level_count(self):
        """The number of levels in the image."""
        return self._osr.level_count

    @property
    def level_dimensions(self):
        """A list of (width, height) tuples, one for each level of the image.

        level_dimensions[n] contains the dimensions of level n."""
        return self._osr.all_level_dimensions

    @property
    def level_downsamples(self):
        """A list of downsampling factors for each level of the image.

        level_downsample[n] contains the downsample factor of level n."""
        return self._osr.all_level_downsample

    @property
    def dimensions(self):
        """A (width, height) tuple for level 0 of the image."""
        return self.level_dimensions[0]

    @property
    def properties(self):
        """Metadata about the image.

        This is a map: property name -> property value."""
        return self._osr.properties

    @property
    def associated_images(self):
        """Images associated with this whole-slide image.

        This is a map: image name -> PIL.Image.

        Unlike in the C interface, the images accessible via this property
        are not premultiplied."""
        return _AssociatedImageMap(self._osr)

    def get_best_level_for_downsample(self, downsample):
        """Return the best level for displaying the given downsample."""
        return self._osr.best_level_for_downsample(downsample)

    def read_region(self, location, level, size):
        """Return a PIL.Image containing the contents of the region.

        location: (x, y) tuple giving the top left pixel in the level 0
                  reference frame.
        level:    the level number.
        size:     (width, height) tuple giving the region size.

        Unlike in the C interface, the image data returned by this
        function is not premultiplied."""
        arr = self._osr.read_region(location, level, size)
        return Image.fromarray(arr)

    def get_thumbnail(self, size):
        """Return a PIL.Image containing an RGB thumbnail of the image.

        size:     the maximum size of the thumbnail."""

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
