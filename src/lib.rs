mod tracer;

use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;

use clap::Parser;

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
    fn trace(&mut self, frame: PyObject, event: tracer::TraceEvent, py: Python) -> PyResult<()> {
        Ok(())
    }
}

/// Runs the command line interface.
#[pyfunction]
fn cli() -> PyResult<()> {
    // TODO: color
    let args = Args::try_parse().map_err(|e| PySystemExit::new_err(e.to_string()))?;
    args.module;
    Ok(())
}

/// Calculate and report region based code coverage.
#[pymodule]
fn fine_coverage(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
