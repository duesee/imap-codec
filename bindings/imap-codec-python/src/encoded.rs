use imap_codec::{
    encode::{Encoded, Fragment},
    imap_types::core::LiteralMode,
};
use pyo3::{prelude::*, types::PyBytes};

/// Python class representing a literal mode
#[derive(Debug, Clone, Copy, PartialEq)]
#[pyclass(name = "LiteralMode", eq)]
pub(crate) enum PyLiteralMode {
    Sync,
    NonSync,
}

impl std::fmt::Display for PyLiteralMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PyLiteralMode::Sync => f.write_str("LiteralMode.Sync"),
            PyLiteralMode::NonSync => f.write_str("LiteralMode.NonSync"),
        }
    }
}

impl From<LiteralMode> for PyLiteralMode {
    fn from(value: LiteralMode) -> Self {
        match value {
            LiteralMode::Sync => PyLiteralMode::Sync,
            LiteralMode::NonSync => PyLiteralMode::NonSync,
        }
    }
}

/// Python class representing a line fragment
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "LineFragment", eq)]
pub(crate) struct PyLineFragment {
    data: Vec<u8>,
}

#[pymethods]
impl PyLineFragment {
    /// Create a new line fragment from data
    ///
    /// `data` can be anything that can be extracted to `Vec`, e.g. Python `bytes`
    #[new]
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Retrieve the data from the fragment as Python `bytes`
    #[getter]
    pub(crate) fn data<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new_bound(py, self.data.as_slice())
    }

    /// String representation of the fragment, e.g. `b'hello'`
    pub(crate) fn __str__(&self, py: Python) -> String {
        self.data(py).to_string()
    }

    /// Printable representation of the fragment, e.g. `LineFragment(b'hello')`
    pub(crate) fn __repr__(&self, py: Python) -> String {
        format!("LineFragment({})", self.__str__(py))
    }
}

/// Python class representing a literal fragment
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "LiteralFragment", eq)]
pub(crate) struct PyLiteralFragment {
    data: Vec<u8>,
    mode: PyLiteralMode,
}

#[pymethods]
impl PyLiteralFragment {
    /// Create a new literal fragment from data and mode
    ///
    /// `data` can be anything that can be extracted to `Vec`, e.g. Python `bytes`
    #[new]
    pub(crate) fn try_new(data: Vec<u8>, mode: PyLiteralMode) -> PyResult<Self> {
        Ok(Self { data, mode })
    }

    /// Retrieve the data of the fragment as Python `bytes`
    #[getter]
    pub(crate) fn data<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new_bound(py, self.data.as_slice())
    }

    /// Retrieve the mode of the fragment
    #[getter]
    pub(crate) fn mode(&self) -> PyLiteralMode {
        self.mode
    }

    /// String representation of the fragment, e.g. `(b'hello', 'Sync')`
    pub(crate) fn __str__(&self, py: Python) -> String {
        format!("({}, {})", self.data(py), self.mode)
    }

    /// Printable representation of the fragment, e.g. `LiteralFragment(b'hello', 'Sync')`
    pub(crate) fn __repr__(&self, py: Python) -> String {
        format!("LiteralFragment{}", self.__str__(py))
    }
}

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
    pub(crate) fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<PyObject>> {
        // Try to get next `Fragment` from `Encoded` iterator
        let Some(fragment) = slf.0.as_mut().and_then(|encoded| encoded.next()) else {
            return Ok(None);
        };

        // Return instance of `PyLineFragment` or `PyLiteralFragment` as a generic `PyObject`.
        Ok(Some(match fragment {
            Fragment::Line { data } => {
                Bound::new(slf.py(), PyLineFragment::new(data))?.to_object(slf.py())
            }
            Fragment::Literal { data, mode } => {
                Bound::new(slf.py(), PyLiteralFragment::try_new(data, mode.into())?)?
                    .to_object(slf.py())
            }
        }))
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
