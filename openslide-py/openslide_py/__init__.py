PROPERTY_NAME_COMMENT = u'openslide.comment'
PROPERTY_NAME_VENDOR = u'openslide.vendor'
PROPERTY_NAME_QUICKHASH1 = u'openslide.quickhash-1'
PROPERTY_NAME_BACKGROUND_COLOR = u'openslide.background-color'
PROPERTY_NAME_OBJECTIVE_POWER = u'openslide.objective-power'
PROPERTY_NAME_MPP_X = u'openslide.mpp-x'
PROPERTY_NAME_MPP_Y = u'openslide.mpp-y'
PROPERTY_NAME_BOUNDS_X = u'openslide.bounds-x'
PROPERTY_NAME_BOUNDS_Y = u'openslide.bounds-y'
PROPERTY_NAME_BOUNDS_WIDTH = u'openslide.bounds-width'
PROPERTY_NAME_BOUNDS_HEIGHT = u'openslide.bounds-height'

from .open_slide import OpenSlide
from .openslide_py import OpenSlideError, OpenSlideUnsupportedFormatError

__all__=["OpenSlide", "OpenSlideError", "OpenSlideUnsupportedFormatError"]
