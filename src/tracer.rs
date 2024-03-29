use pyo3::{Python,ffi};

pub(crate) fn install() {
    Python::with_gil(|py| {
        /* Safty: see https://pyo3.rs/main/doc/pyo3/ffi/index.html/#safety and https://docs.python.org/3/c-api/init.html#c.PyEval_SetTrace
        * Second argument can be null, is intended to hold state per thread
        * Caller must hold GIL
        */
        unsafe {
            ffi::PyEval_SetTrace(Some(py_trace_callback), std::ptr::null_mut());
        }
    });
}

pub(crate) enum Arg {
    Call,
    Exception { typ: pyo3::PyObject, value: pyo3::PyObject, traceback: pyo3::PyObject },
    Line,
    Return(pyo3::PyObject),
    Opcode,
}

impl TryFrom<(i32, *mut ffi::PyObject)> for Arg {
    type Error = ();

    // See https://docs.python.org/3/c-api/init.html#c.PyEval_SetTrace
    // Any trace function registered using PyEval_SetTrace() will not receive PyTrace_C_CALL, PyTrace_C_EXCEPTION or PyTrace_C_RETURN as a value for the what parameter.
    fn try_from(value: (i32, *mut pyo3::ffi::PyObject)) -> Result<Self, Self::Error> {
        let (what, arg) = value;
        Ok(match what { // determines type of `arg`
            ffi::PyTrace_CALL => Arg::Call, // Always Py_None.
            ffi::PyTrace_EXCEPTION => Arg::Exception { typ: todo!(), value: todo!(), traceback: todo!() }, // Exception information as returned by sys.exc_info().
            ffi::PyTrace_LINE => Arg::Line, // Always Py_None.
            ffi::PyTrace_RETURN => Arg::Return(todo!()), // Value being returned to the caller, or NULL if caused by an exception.
            ffi::PyTrace_OPCODE => Arg::Opcode, // Always Py_None.
            _ => return Err(()),
        })
    }
}

/// See https://docs.python.org/3/c-api/init.html#c.Py_tracefunc
unsafe extern "C" fn py_trace_callback(obj: *mut ffi::PyObject, frame: *mut ffi::PyFrameObject, what: i32, arg: *mut ffi::PyObject) -> i32 {
    let arg: Arg = (what, arg).try_into().expect("invalid argument");
    0
}
