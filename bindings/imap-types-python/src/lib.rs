use imap_codec::{encode::Encoder, GreetingCodec};
use imap_types::response::Greeting;
use pyo3::prelude::*;

#[pyclass(name = "Greeting")]
struct PyGreeting(Greeting<'static>);

#[pymethods]
impl PyGreeting {
    #[new]
    pub fn new(py: Python, object: PyObject) -> PyResult<Self> {
        let x = serde_pyobject::from_pyobject(object.into_bound(py))?;
        Ok(Self(x))
    }

    fn __repr__(&self, py: Python) -> String {
        let obj = serde_pyobject::to_pyobject(py, &self.0).unwrap();
        format!("Greeting({:?})", obj)
    }

    fn __str__(&self) -> String {
        let codec = GreetingCodec::new();

        // TODO: * Clarify if we want to use `imap-codec` already
        //       * Resolve `unwrap()`
        format!(
            "{}",
            String::from_utf8(codec.encode(&self.0).dump()).unwrap()
        )
    }
}

#[pymodule]
#[pyo3(name = "imap_types")]
fn imap_types_python(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyGreeting>()?;

    Ok(())
}
