use pyo3::prelude::*;

use pyo3::types::PyFrame;

use crate::tracer::{TraceEvent, Tracer};

#[pyclass]
#[derive(Default)]
pub struct Collector {}

impl Tracer<TraceEvent> for Collector {
    fn trace(&mut self, frame: Py<PyFrame>, event: TraceEvent, py: Python) -> PyResult<()> {
        let frame = frame.into_bound(py);
        match event {
            TraceEvent::Exception {
                exc_type,
                exc_value,
                exc_traceback,
            } => {
                dbg!(
                    "exception",
                    frame,
                    exc_type.into_bound(py),
                    exc_value.into_bound(py),
                    exc_traceback.into_bound(py)
                );
            }
            TraceEvent::Return(value) => {
                let value = value.map(|value| value.into_bound(py));
                dbg!("return", frame, value);
            }
            _ => {
                dbg!(event, frame);
            }
        }
        Ok(())
    }
}
