use std::ffi::c_int;
use std::fmt::Debug;

use pyo3::exceptions::PyValueError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::pyclass::boolean_struct::False;
use pyo3::types::PyFrame;
use pyo3::PyClass;

/// Wrap pyo3-ffi/src/cpython/pystate.rs#L18-L25
#[derive(Debug)]
pub enum TraceEvent {
    Call,
    Exception {
        exc_type: PyObject,
        exc_value: PyObject,
        exc_traceback: PyObject,
    },
    Line,
    Return(Option<PyObject>), // TODO: PyResult instead?
    Opcode,
}

#[derive(Debug)]
pub enum ProfileEvent {
    Call,
    Return(Option<PyObject>), // TODO: PyResult instead?
    CCall,
    CException, // TODO
    CReturn,    // TODO
}

pub trait Event: Sized + Debug {
    fn from_raw<'py>(what: c_int, arg: Option<Bound<'py, PyAny>>) -> PyResult<Self>;
}

impl Event for TraceEvent {
    fn from_raw<'py>(what: c_int, arg: Option<Bound<'py, PyAny>>) -> PyResult<Self> {
        Ok(match (what, arg) {
            (ffi::PyTrace_CALL, _) => Self::Call,
            (ffi::PyTrace_EXCEPTION, value) => {
                let (exc_type, exc_value, exc_traceback) = value
                    .ok_or_else(|| PyValueError::new_err("PyTrace_EXCEPTION without exc_info"))?
                    .extract::<(PyObject, PyObject, PyObject)>()?;
                Self::Exception {
                    exc_type,
                    exc_value,
                    exc_traceback,
                }
            }
            (ffi::PyTrace_LINE, _) => Self::Line,
            (ffi::PyTrace_RETURN, value) => Self::Return(value.map(Bound::unbind)),
            (ffi::PyTrace_OPCODE, _) => Self::Opcode,
            (what, _) => {
                return Err(PyValueError::new_err(format!(
                    "invalid trace event type {what}"
                )))
            }
        })
    }
}

impl Event for ProfileEvent {
    fn from_raw<'py>(what: c_int, arg: Option<Bound<'py, PyAny>>) -> PyResult<Self> {
        Ok(match (what, arg) {
            (ffi::PyTrace_CALL, _) => Self::Call,
            (ffi::PyTrace_RETURN, value) => Self::Return(value.map(Bound::unbind)),
            (ffi::PyTrace_C_CALL, _) => Self::CCall,
            (ffi::PyTrace_C_EXCEPTION, _) => Self::CException,
            (ffi::PyTrace_C_RETURN, _) => Self::CReturn,
            (what, _) => {
                return Err(PyValueError::new_err(format!(
                    "invalid profile event type {what}"
                )))
            }
        })
    }
}

macro_rules! try_py {
    ($py:ident, $($arg:tt)*) => {
        match $($arg)* {
            Ok(val) => val,
            Err(err) => {
                err.restore($py);
                return -1;
            }
        }
    };
}

extern "C" fn trace_func<E, T>(
    obj: *mut ffi::PyObject,
    frame: *mut ffi::PyFrameObject,
    what: c_int,
    arg: *mut ffi::PyObject,
) -> c_int
where
    E: Event,
    T: Tracer<E>,
{
    // Safety:
    //
    // `frame` is an `ffi::PyFrameObject` which can be converted safely to a `PyObject`.
    let frame = frame as *mut ffi::PyObject;
    Python::with_gil(|py| {
        // Safety:
        //
        // `obj` is a reference to our `Profiler` wrapped up in a Python object, so
        // we can safely convert it from an `ffi::PyObject` to a `PyObject`.
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.from_borrowed_ptr_or_err
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let obj = try_py!(py, unsafe { PyObject::from_borrowed_ptr_or_err(py, obj) });
        let mut tracer = try_py!(py, obj.extract::<PyRefMut<T>>(py));

        // Safety:
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.from_borrowed_ptr_or_err
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let frame = try_py!(py, unsafe { PyObject::from_borrowed_ptr_or_err(py, frame) });
        let frame = try_py!(py, frame.extract(py));

        // Safety:
        //
        // `arg` is either a `Py_None` (PyTrace_CALL) or any PyObject (PyTrace_RETURN) or
        // NULL (PyTrace_RETURN).
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.from_borrowed_ptr_or_opt
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let arg = unsafe { PyObject::from_borrowed_ptr_or_opt(py, arg) };
        // `arg` is `NULL` when the frame exits with an exception unwinding instead of a normal return.
        // So it might be possible to make `arg` a `PyResult` here instead of an option, but I haven't worked out the detail of how that would work.

        let event = try_py!(py, E::from_raw(what, arg.map(|arg| arg.into_bound(py))));

        try_py!(py, tracer.trace(frame, event, py));
        0
    })
}

pub trait Tracer<E>: PyClass<Frozen = False>
where
    E: Event,
{
    fn trace(&mut self, frame: Py<PyFrame>, event: E, py: Python) -> PyResult<()>;
}

pub trait Register<E: Event> {
    fn register(self) -> PyResult<()>;
    fn deregister(self) -> PyResult<()>;
}

impl<'a, T> Register<ProfileEvent> for Bound<'a, T>
where
    T: Tracer<ProfileEvent>,
{
    fn register(self) -> PyResult<()> {
        let py = self.py();
        unsafe {
            ffi::PyEval_SetProfile(Some(trace_func::<ProfileEvent, T>), self.into_ptr());
        }
        match PyErr::take(py) {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }
    fn deregister(self) -> PyResult<()> {
        unsafe {
            ffi::PyEval_SetProfile(None, self.into_ptr());
        }
        Ok(())
    }
}

impl<'py, T> Register<TraceEvent> for Bound<'py, T>
where
    T: Tracer<TraceEvent>,
{
    fn register(self) -> PyResult<()> {
        let py = self.py();
        unsafe {
            ffi::PyEval_SetTrace(Some(trace_func::<TraceEvent, T>), self.into_ptr());
        }
        match PyErr::take(py) {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }
    fn deregister(self) -> PyResult<()> {
        unsafe {
            ffi::PyEval_SetTrace(None, self.into_ptr());
        }
        Ok(())
    }
}
