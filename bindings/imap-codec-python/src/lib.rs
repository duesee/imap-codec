mod encoded;

use encoded::PyEncoded;
use imap_codec::{
    decode::{self, Decoder},
    encode::Encoder,
    AuthenticateDataCodec, CommandCodec, GreetingCodec, IdleDoneCodec, ResponseCodec,
};
use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyBytes};

// Create exception types for decode errors
create_exception!(imap_codec, DecodeError, PyException);
create_exception!(imap_codec, DecodeFailed, DecodeError);
create_exception!(imap_codec, DecodeIncomplete, DecodeError);
create_exception!(imap_codec, DecodeLiteralFound, DecodeError);

/// Python class for using `GreetingCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "GreetingCodec")]
struct PyGreetingCodec;

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

    /// Encode greeting into fragments
    #[staticmethod]
    fn encode(greeting: Bound<PyAny>) -> PyResult<PyEncoded> {
        let greeting = serde_pyobject::from_pyobject(greeting)?;
        let encoded = GreetingCodec::default().encode(&greeting);
        Ok(PyEncoded(Some(encoded)))
    }
}

/// Python class for using `CommandCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "CommandCodec")]
struct PyCommandCodec;

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

    /// Encode command into fragments
    #[staticmethod]
    fn encode(command: Bound<PyAny>) -> PyResult<PyEncoded> {
        let command = serde_pyobject::from_pyobject(command)?;
        let encoded = CommandCodec::default().encode(&command);
        Ok(PyEncoded(Some(encoded)))
    }
}

/// Python class for using `AuthenticateDataCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "AuthenticateDataCodec")]
struct PyAuthenticateDataCodec;

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

    /// Encode authenticate data line into fragments
    #[staticmethod]
    fn encode(authenticate_data: Bound<PyAny>) -> PyResult<PyEncoded> {
        let authenticate_data = serde_pyobject::from_pyobject(authenticate_data)?;
        let encoded = AuthenticateDataCodec::default().encode(&authenticate_data);
        Ok(PyEncoded(Some(encoded)))
    }
}

/// Python class for using `ResponseCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "ResponseCodec")]
struct PyResponseCodec;

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

    /// Encode response into fragments
    #[staticmethod]
    fn encode(response: Bound<PyAny>) -> PyResult<PyEncoded> {
        let response = serde_pyobject::from_pyobject(response)?;
        let encoded = ResponseCodec::default().encode(&response);
        Ok(PyEncoded(Some(encoded)))
    }
}

/// Python class for using `IdleDoneCodec`
#[derive(Debug, Clone, PartialEq)]
#[pyclass(name = "IdleDoneCodec")]
struct PyIdleDoneCodec;

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

    /// Encode idle done into fragments
    #[staticmethod]
    fn encode(idle_done: Bound<PyAny>) -> PyResult<PyEncoded> {
        let idle_done = serde_pyobject::from_pyobject(idle_done)?;
        let encoded = IdleDoneCodec::default().encode(&idle_done);
        Ok(PyEncoded(Some(encoded)))
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
    m.add_class::<encoded::PyLiteralMode>()?;
    m.add_class::<encoded::PyLineFragment>()?;
    m.add_class::<encoded::PyLiteralFragment>()?;
    m.add_class::<PyEncoded>()?;
    m.add_class::<PyGreetingCodec>()?;
    m.add_class::<PyCommandCodec>()?;
    m.add_class::<PyAuthenticateDataCodec>()?;
    m.add_class::<PyResponseCodec>()?;
    m.add_class::<PyIdleDoneCodec>()?;

    Ok(())
}
