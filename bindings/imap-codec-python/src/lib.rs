use imap_codec::{
    decode::{self, Decoder},
    AuthenticateDataCodec, CommandCodec, GreetingCodec, IdleDoneCodec, ResponseCodec,
};
use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyBytes};

// Create exception types for decode errors
create_exception!(imap_codec, DecodeError, PyException);
create_exception!(imap_codec, DecodeFailed, DecodeError);
create_exception!(imap_codec, DecodeIncomplete, DecodeError);
create_exception!(imap_codec, DecodeLiteralFound, DecodeError);

/// Wrapper for `GreetingCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "GreetingCodec")]
struct PyGreetingCodec(GreetingCodec);

#[pymethods]
impl PyGreetingCodec {
    /// Decode greeting from given bytes
    #[staticmethod]
    fn decode(bytes: Bound<PyBytes>) -> PyResult<(Bound<PyBytes>, Bound<PyAny>)> {
        let py = bytes.py();
        let (remaining, greeting) =
            GreetingCodec::default()
                .decode(bytes.as_bytes())
                .map_err(|e| match e {
                    decode::GreetingDecodeError::Incomplete => DecodeIncomplete::new_err(()),
                    decode::GreetingDecodeError::Failed => DecodeFailed::new_err(()),
                })?;
        Ok((
            PyBytes::new_bound(py, remaining),
            serde_pyobject::to_pyobject(py, &greeting)?,
        ))
    }
}

/// Wrapper for `CommandCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "CommandCodec")]
struct PyCommandCodec(CommandCodec);

#[pymethods]
impl PyCommandCodec {
    /// Decode command from given bytes
    #[staticmethod]
    fn decode(bytes: Bound<PyBytes>) -> PyResult<(Bound<PyBytes>, Bound<PyAny>)> {
        let py = bytes.py();
        match CommandCodec::default().decode(bytes.as_bytes()) {
            Ok((remaining, command)) => Ok((
                PyBytes::new_bound(py, remaining),
                serde_pyobject::to_pyobject(py, &command)?,
            )),
            Err(err) => Err(match err {
                decode::CommandDecodeError::Incomplete => DecodeIncomplete::new_err(()),
                decode::CommandDecodeError::LiteralFound { tag, length, mode } => {
                    let dict = pyo3::types::PyDict::new_bound(py);
                    dict.set_item("tag", serde_pyobject::to_pyobject(py, &tag)?)?;
                    dict.set_item("length", length)?;
                    dict.set_item("mode", serde_pyobject::to_pyobject(py, &mode)?)?;
                    DecodeLiteralFound::new_err(dict.unbind())
                }
                decode::CommandDecodeError::Failed => DecodeFailed::new_err(()),
            }),
        }
    }
}

/// Wrapper for `AuthenticateDataCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "AuthenticateDataCodec")]
struct PyAuthenticateDataCodec(AuthenticateDataCodec);

#[pymethods]
impl PyAuthenticateDataCodec {
    /// Decode authenticate data line from given bytes
    #[staticmethod]
    fn decode(bytes: Bound<PyBytes>) -> PyResult<(Bound<PyBytes>, Bound<PyAny>)> {
        let py = bytes.py();
        match AuthenticateDataCodec::default().decode(bytes.as_bytes()) {
            Ok((remaining, authenticate_data)) => Ok((
                PyBytes::new_bound(py, remaining),
                serde_pyobject::to_pyobject(py, &authenticate_data)?,
            )),
            Err(err) => Err(match err {
                decode::AuthenticateDataDecodeError::Incomplete => DecodeIncomplete::new_err(()),
                decode::AuthenticateDataDecodeError::Failed => DecodeFailed::new_err(()),
            }),
        }
    }
}

/// Wrapper for `ResponseCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "ResponseCodec")]
struct PyResponseCodec(ResponseCodec);

#[pymethods]
impl PyResponseCodec {
    /// Decode response from given bytes
    #[staticmethod]
    fn decode(bytes: Bound<PyBytes>) -> PyResult<(Bound<PyBytes>, Bound<PyAny>)> {
        let py = bytes.py();
        match ResponseCodec::default().decode(bytes.as_bytes()) {
            Ok((remaining, response)) => Ok((
                PyBytes::new_bound(py, remaining),
                serde_pyobject::to_pyobject(py, &response)?,
            )),
            Err(err) => Err(match err {
                decode::ResponseDecodeError::Incomplete => DecodeIncomplete::new_err(()),
                decode::ResponseDecodeError::LiteralFound { length } => {
                    let dict = pyo3::types::PyDict::new_bound(py);
                    dict.set_item("length", length)?;
                    DecodeLiteralFound::new_err(dict.unbind())
                }
                decode::ResponseDecodeError::Failed => DecodeFailed::new_err(()),
            }),
        }
    }
}

/// Wrapper for `IdleDoneCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "IdleDoneCodec")]
struct PyIdleDoneCodec(IdleDoneCodec);

#[pymethods]
impl PyIdleDoneCodec {
    /// Decode idle done from given bytes
    #[staticmethod]
    fn decode(bytes: Bound<PyBytes>) -> PyResult<(Bound<PyBytes>, Bound<PyAny>)> {
        let py = bytes.py();
        match IdleDoneCodec::default().decode(bytes.as_bytes()) {
            Ok((remaining, idle_done)) => Ok((
                PyBytes::new_bound(py, remaining),
                serde_pyobject::to_pyobject(py, &idle_done)?,
            )),
            Err(err) => Err(match err {
                decode::IdleDoneDecodeError::Incomplete => DecodeIncomplete::new_err(()),
                decode::IdleDoneDecodeError::Failed => DecodeFailed::new_err(()),
            }),
        }
    }
}

#[pymodule]
#[pyo3(name = "imap_codec")]
fn imap_codec_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("DecodeError", m.py().get_type_bound::<DecodeError>())?;
    m.add("DecodeFailed", m.py().get_type_bound::<DecodeFailed>())?;
    m.add(
        "DecodeIncomplete",
        m.py().get_type_bound::<DecodeIncomplete>(),
    )?;
    m.add(
        "DecodeLiteralFound",
        m.py().get_type_bound::<DecodeLiteralFound>(),
    )?;
    m.add_class::<PyGreetingCodec>()?;
    m.add_class::<PyCommandCodec>()?;
    m.add_class::<PyAuthenticateDataCodec>()?;
    m.add_class::<PyResponseCodec>()?;
    m.add_class::<PyIdleDoneCodec>()?;

    Ok(())
}
