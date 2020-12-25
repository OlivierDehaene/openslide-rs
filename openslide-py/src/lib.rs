use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, PyClass};
use std::error::Error;
use std::path::Path;

use ndarray_image::{NdColor, NdImage};
use numpy::{IntoPyArray, PyArray3};
use openslide_rs;
use pyo3::callback::IntoPyCallbackOutput;
use pyo3::class::context::{PyContextEnterProtocol, PyContextExitProtocol, PyContextProtocol};
use pyo3::types::PyType;

#[pyclass]
struct _OpenSlide {
    inner: openslide_rs::OpenSlide,
}

// #[pyproto]
// impl<'p> PyContextProtocol<'p> for OpenSlide {
//     fn __enter__(&'p mut self) -> PyResult<&mut OpenSlide> {
//         Ok(self)
//     }
//
//     fn __exit__(
//         &mut self,
//         ty: Option<&'p PyType>,
//         _value: Option<&'p PyAny>,
//         _traceback: Option<&'p PyAny>,
//     ) -> PyResult<()> {
//         Ok(())
//     }
// }

#[pymethods]
impl _OpenSlide {
    #[new]
    fn new(filename: &str) -> Self {
        _OpenSlide {
            inner: openslide_rs::OpenSlide::open(Path::new(filename)).unwrap(),
        }
    }

    pub fn level_dimensions(&self, level: u32) -> (u64, u64) {
        self.inner.level_dimensions(level).unwrap()
    }

    pub fn read_region<'py>(
        &self,
        py: Python<'py>,
        address: (u32, u32),
        level: u32,
        size: (u32, u32),
    ) -> &'py PyArray3<u8> {
        let region = self
            .inner
            .read_region(
                openslide_rs::Address::new(address.0, address.1),
                level,
                openslide_rs::Size::new(size.0, size.1),
            )
            .unwrap();
        let region: NdColor = NdImage(&region).into();
        region.to_owned().into_pyarray(py)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn openslide_py(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<_OpenSlide>()?;

    Ok(())
}
