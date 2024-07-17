use imap_codec::encode::Encoded;
use pyo3::{prelude::*, types::PyBytes};

/// Python wrapper classes for `Encoded`
///
/// This implements a Python iterator over the containing fragments.
#[derive(Debug, Clone)]
#[pyclass(name = "Encoded")]
pub(crate) struct PyEncoded(pub(crate) Option<Encoded>);

#[pymethods]
impl PyEncoded {
    /// Initialize iterator
    pub(crate) fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Return next fragment
    pub(crate) fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<Bound<PyAny>>> {
        let Some(encoded) = &mut slf.0 else {
            return Ok(None);
        };
        Ok(encoded
            .next()
            .map(|value| serde_pyobject::to_pyobject(slf.py(), &value))
            .transpose()?)
    }

    /// Dump remaining fragment data
    pub(crate) fn dump(mut slf: PyRefMut<'_, Self>) -> PyResult<Bound<PyBytes>> {
        let encoded = slf.0.take();
        let dump = match encoded {
            Some(encoded) => encoded.dump(),
            None => Vec::new(),
        };
        Ok(PyBytes::new_bound(slf.py(), &dump))
    }
}
