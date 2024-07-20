use imap_codec::imap_types::{
    auth::AuthenticateData,
    command::Command,
    extensions::idle::IdleDone,
    response::{Greeting, Response},
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyString},
};

/// Python wrapper class around `Greeting`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "Greeting", eq)]
pub(crate) struct PyGreeting(pub(crate) Greeting<'static>);

#[pymethods]
impl PyGreeting {
    /// Deserialize greeting from dictionary
    #[staticmethod]
    pub(crate) fn from_dict(greeting: Bound<PyDict>) -> PyResult<Self> {
        Ok(Self(serde_pyobject::from_pyobject(greeting)?))
    }

    /// Serialize greeting into dictionary
    pub(crate) fn as_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        Ok(serde_pyobject::to_pyobject(py, &self.0)?.downcast_into()?)
    }

    pub(crate) fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("Greeting({})", self.as_dict(py)?))
    }
}

/// Python wrapper class around `Command`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "Command", eq)]
pub(crate) struct PyCommand(pub(crate) Command<'static>);

#[pymethods]
impl PyCommand {
    /// Deserialize command from dictionary
    #[staticmethod]
    pub(crate) fn from_dict(command: Bound<PyDict>) -> PyResult<Self> {
        Ok(Self(serde_pyobject::from_pyobject(command)?))
    }

    /// Serialize command into dictionary
    pub(crate) fn as_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        Ok(serde_pyobject::to_pyobject(py, &self.0)?.downcast_into()?)
    }

    pub(crate) fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("Command({:?})", self.as_dict(py)?))
    }
}

/// Python wrapper class around `AuthenticateData`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "AuthenticateData", eq)]
pub(crate) struct PyAuthenticateData(pub(crate) AuthenticateData<'static>);

#[pymethods]
impl PyAuthenticateData {
    /// Deserialize authenticate data line from dictionary
    #[staticmethod]
    pub(crate) fn from_dict(authenticate_data: Bound<PyDict>) -> PyResult<Self> {
        Ok(Self(serde_pyobject::from_pyobject(authenticate_data)?))
    }

    /// Serialize authenticate data line into dictionary
    pub(crate) fn as_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let object = serde_pyobject::to_pyobject(py, &self.0)?;
        Ok(if object.is_instance_of::<PyString>() {
            // Unit variants are deserialized into strings by `to_pyobject`, create a
            // dictionary around it for a consistent interface: `{ "Variant": {} }`
            let dict = PyDict::new_bound(py);
            dict.set_item(object, PyDict::new_bound(py))?;
            dict
        } else {
            object.downcast_into()?
        })
    }

    pub(crate) fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("AuthenticateData({:?})", self.as_dict(py)?))
    }
}

/// Python wrapper class around `Response`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "Response", eq)]
pub(crate) struct PyResponse(pub(crate) Response<'static>);

#[pymethods]
impl PyResponse {
    /// Deserialize response from dictionary
    #[staticmethod]
    pub(crate) fn from_dict(response: Bound<PyDict>) -> PyResult<Self> {
        Ok(Self(serde_pyobject::from_pyobject(response)?))
    }

    /// Serialize response into dictionary
    pub(crate) fn as_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        Ok(serde_pyobject::to_pyobject(py, &self.0)?.downcast_into()?)
    }

    pub(crate) fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("Response({:?})", self.as_dict(py)?))
    }
}

/// Python wrapper class around `IdleDone`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "IdleDone", eq)]
pub(crate) struct PyIdleDone(pub(crate) IdleDone);

#[pymethods]
impl PyIdleDone {
    #[new]
    pub(crate) fn new() -> Self {
        Self(IdleDone)
    }

    pub(crate) fn __repr__(&self) -> &str {
        "IdleDone"
    }
}
