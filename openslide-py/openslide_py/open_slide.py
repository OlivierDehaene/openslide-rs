from PIL import Image

from .openslide_py import _OpenSlide


class OpenSlide:
    def __init__(self, filename):
        self._osr = _OpenSlide(filename)

    def read_region(self):
        region_arr = self._osr.read_region((0, 0), 0, (2000, 2000))
        return Image.fromarray(region_arr)
