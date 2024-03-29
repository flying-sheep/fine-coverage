use std::ffi::c_int;

use pyo3::prelude::*;
use pyo3::ffi;
use pyo3::PyClass;
use pyo3::pyclass::boolean_struct::False;

/// Wrap pyo3-ffi/src/cpython/pystate.rs#L18-L25
pub enum TraceEvent {
    Call,
    Exception,
    Line,
    Return,
    Opcode,
}

impl TryFrom<c_int> for TraceEvent {
    type Error = ();
    fn try_from(what: c_int) -> Result<Self, Self::Error> {
        Ok(match what {
            ffi::PyTrace_CALL => Self::Call,
            ffi::PyTrace_EXCEPTION => Self::Exception,
            ffi::PyTrace_LINE => Self::Line,
            ffi::PyTrace_RETURN => Self::Return,
            ffi::PyTrace_OPCODE => Self::Opcode,
            _ => return Err(()),
        })
    }
}

pub enum ProfileEvent {
    TraceEvent(TraceEvent),
    CCall,
    CException,
    CReturn,
}

impl From<TraceEvent> for ProfileEvent {
    fn from(event: TraceEvent) -> Self {
        Self::TraceEvent(event)
    }
}

impl TryFrom<c_int> for ProfileEvent {
    type Error = ();
    fn try_from(what: c_int) -> Result<Self, Self::Error> {
        if let Ok(event) = TraceEvent::try_from(what) {
            return Ok(Self::from(event));
        }
        Ok(match what {
            ffi::PyTrace_C_CALL => Self::CCall,
            ffi::PyTrace_C_EXCEPTION => Self::CException,
            ffi::PyTrace_C_RETURN => Self::CReturn,
            _ => return Err(()),
        })
    }
}

pub trait Event: TryFrom<c_int, Error = ()> {}
impl Event for TraceEvent {}
impl Event for ProfileEvent {}

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

extern "C" fn trace_func<E, P>(
    _obj: *mut ffi::PyObject,
    _frame: *mut ffi::PyFrameObject,
    what: c_int,
    _arg: *mut ffi::PyObject,
) -> c_int where P: Tracer<E>, E: Event {
    let _frame = _frame as *mut ffi::PyObject;
    Python::with_gil(|py| {
        // Safety:
        //
        // `from_borrowed_ptr_or_err` must be called in an unsafe block.
        //
        // `_obj` is a reference to our `Profiler` wrapped up in a Python object, so
        // we can safely convert it from an `ffi::PyObject` to a `PyObject`.
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.from_borrowed_ptr_or_err
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let obj = try_py!(py, unsafe { PyObject::from_borrowed_ptr_or_err(py, _obj) });
        let mut tracer = try_py!(py, obj.extract::<PyRefMut<P>>(py));

        // Safety:
        //
        // `from_borrowed_ptr_or_err` must be called in an unsafe block.
        //
        // `_frame` is an `ffi::PyFrameObject` which can be converted safely
        // to a `PyObject`. We can later convert it into a `pyo3::types::PyFrame`.
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.from_borrowed_ptr_or_err
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let frame = try_py!(py, unsafe { PyObject::from_borrowed_ptr_or_err(py, _frame) });

        // Safety:
        //
        // `from_borrowed_ptr_or_opt` must be called in an unsafe block.
        //
        // `_arg` is either a `Py_None` (PyTrace_CALL) or any PyObject (PyTrace_RETURN) or
        // NULL (PyTrace_RETURN).
        //
        // We borrow the object so we don't break reference counting.
        //
        // https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.from_borrowed_ptr_or_opt
        // https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
        let arg = unsafe { PyObject::from_borrowed_ptr_or_opt(py, _arg) };
        // `_arg` is `NULL` when the frame exits with an exception unwinding instead of a normal return.
        // So it might be possible to make `arg` a `PyResult` here instead of an option, but I haven't worked out the detail of how that would work.

        let event = E::try_from(what).expect("invalid `what`");

        try_py!(py, tracer.trace(frame, arg, event, py));
        0
    })
}

pub trait Tracer<E>: PyClass<Frozen = False> where E: Event {
    fn trace(
        &mut self,
        frame: PyObject,
        arg: Option<PyObject>,
        event: E,
        py: Python,
    ) -> PyResult<()>;
}

pub trait Register<E: Event> {
    fn register(self) -> PyResult<()>;
    fn deregister(self) -> PyResult<()>;
}

impl<'a, P> Register<ProfileEvent> for Bound<'a, P> where P: Tracer<ProfileEvent> {
    fn register(self) -> PyResult<()> {
        let py = self.py();
        unsafe {
            ffi::PyEval_SetProfile(Some(trace_func::<ProfileEvent, P>), self.into_ptr());
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

impl<'a, P> Register<TraceEvent> for Bound<'a, P> where P: Tracer<TraceEvent> {
    fn register(self) -> PyResult<()> {
        let py = self.py();
        unsafe {
            ffi::PyEval_SetTrace(Some(trace_func::<TraceEvent, P>), self.into_ptr());
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
