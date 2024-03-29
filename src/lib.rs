mod tracer;

use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;

use clap::Parser;
use pyo3::types::PyFrame;

use crate::tracer::Register;

/// Program to calculate and report region based code coverage.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Module or path to run
    name: String,

    /// Treat `name` as module name
    #[arg(short, long)]
    module: bool,

    /// Module to calculate coverage for
    #[arg(short, long)]
    cov: Option<String>,
}

#[pyclass]
struct Tracer;

impl tracer::Tracer<tracer::TraceEvent> for Tracer {
    fn trace(&mut self, frame: Py<PyFrame>, event: tracer::TraceEvent, py: Python) -> PyResult<()> {
        let frame = frame.into_bound(py);
        match event {
            tracer::TraceEvent::Exception {
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
            tracer::TraceEvent::Return(value) => {
                dbg!("return", frame, value.map(|value| value.into_bound(py)));
            }
            _ => {
                dbg!(event, frame);
            }
        }
        Ok(())
    }
}

/// Runs the command line interface.
#[pyfunction]
fn cli() -> PyResult<()> {
    // TODO: color
    let args = Args::try_parse().map_err(|e| PySystemExit::new_err(e.to_string()))?;
    let tracer = Tracer {};
    Python::with_gil(|py| Bound::new(py, tracer)?.register())?;
    // TODO: run module
    args.module;
    Ok(())
}

/// Calculate and report region based code coverage.
#[pymodule]
fn fine_coverage(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
