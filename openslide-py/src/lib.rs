use pyo3::exceptions::{PyFileNotFoundError, PyIndexError, PyKeyError};
use pyo3::prelude::*;

use std::path::Path;

use ndarray_image::{NdColor, NdImage};
use numpy::{IntoPyArray, PyArray3};

use pyo3::create_exception;
use pyo3::exceptions::PyException;

use pyo3::types::PyType;

create_exception!(openslide_py, OpenSlideError, PyException);
create_exception!(openslide_py, OpenSlideUnsupportedFormatError, PyException);

fn match_error(error: openslide_rs::OpenSlideError) -> PyErr {
    match error {
        openslide_rs::OpenSlideError::MissingFile(m) => PyFileNotFoundError::new_err(m),
        openslide_rs::OpenSlideError::UnsupportedFile(m) => {
            OpenSlideUnsupportedFormatError::new_err(m)
        }
        openslide_rs::OpenSlideError::KeyError(m) => PyKeyError::new_err(m),
        openslide_rs::OpenSlideError::IndexError(m) => PyIndexError::new_err(m),
        openslide_rs::OpenSlideError::InternalError(m) => OpenSlideError::new_err(m),
    }
}

#[pyclass]
struct _OpenSlide {
    inner: openslide_rs::OpenSlide,
}

#[pymethods]
impl _OpenSlide {
    #[classmethod]
    fn detect_format(_cls: &PyType, filename: &str) -> PyResult<String> {
        openslide_rs::OpenSlide::detect_vendor(Path::new(filename)).map_err(match_error)
    }

    #[new]
    fn new(filename: &str) -> PyResult<Self> {
        let inner = openslide_rs::OpenSlide::open(Path::new(filename)).map_err(match_error)?;
        Ok(_OpenSlide { inner })
    }

    fn level_dimensions(&self, level: u32) -> PyResult<(u64, u64)> {
        let openslide_rs::Size { w, h } =
            self.inner.level_dimensions(level).map_err(match_error)?;
        Ok((w as u64, h as u64))
    }

    fn level_downsample(&self, level: u32) -> PyResult<f64> {
        self.inner.level_downsample(level).map_err(match_error)
    }

    fn best_level_for_downsample(&self, downsample: f64) -> PyResult<u32> {
        self.inner
            .best_level_for_downsample(downsample)
            .map_err(match_error)
    }

    fn property(&self, name: &str) -> PyResult<String> {
        self.inner.property(name).map_err(match_error)
    }

    fn associated_image<'py>(&self, py: Python<'py>, name: &str) -> PyResult<&'py PyArray3<u8>> {
        let image = self.inner.associated_image(name).map_err(match_error)?;
        let image: NdColor = NdImage(&image).into();
        Ok(image.to_owned().into_pyarray(py))
    }

    #[getter]
    fn level_count(&self) -> PyResult<u32> {
        self.inner.level_count().map_err(match_error)
    }

    #[getter]
    fn all_level_dimensions(&self) -> PyResult<Vec<(u64, u64)>> {
        let dimensions = (0..self.level_count()?)
            .map(|level| self.level_dimensions(level).unwrap())
            .collect();
        Ok(dimensions)
    }

    #[getter]
    fn all_level_downsample(&self) -> PyResult<Vec<f64>> {
        let dimensions = (0..self.level_count()?)
            .map(|level| self.level_downsample(level).unwrap())
            .collect();
        Ok(dimensions)
    }

    #[getter]
    fn property_names(&self) -> PyResult<Vec<String>> {
        self.inner.property_names().map_err(match_error)
    }

    #[getter]
    fn associated_image_names(&self) -> PyResult<Vec<String>> {
        self.inner.associated_image_names().map_err(match_error)
    }

    fn set_cache_size(&self, cache_size: u32) -> PyResult<()> {
        self.inner.set_cache_size(cache_size).map_err(match_error)
    }

    fn read_region<'py>(
        &self,
        py: Python<'py>,
        address: (u32, u32),
        level: u32,
        size: (u32, u32),
    ) -> PyResult<&'py PyArray3<u8>> {
        let region_coordinates = openslide_rs::Region {
            address: openslide_rs::Address::from(address),
            level: level as _,
            size: openslide_rs::Size::from(size),
        };
        let region = self
            .inner
            .read_region(region_coordinates)
            .map_err(match_error)?;
        let region: NdColor = NdImage(&region).into();
        Ok(region.to_owned().into_pyarray(py))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn openslide_py(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<_OpenSlide>()?;
    m.add("OpenSlideError", py.get_type::<OpenSlideError>())?;
    m.add(
        "OpenSlideUnsupportedFormatError",
        py.get_type::<OpenSlideUnsupportedFormatError>(),
    )?;

    Ok(())
}
