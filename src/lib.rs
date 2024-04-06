#![warn(clippy::pedantic)]

mod collector;
mod tracer;

use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;

use clap::Parser;

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

    #[arg(trailing_var_arg(true))]
    options: Vec<String>,
}

/// Runs the command line interface.
#[pyfunction]
fn cli() -> PyResult<()> {
    // TODO: only parse non-interpreter args (e.g. this breaks when run via `python -m`)
    let args = Args::try_parse_from(std::env::args_os().skip(1)).map_err(|e| {
        use clap::error::ErrorKind::{DisplayHelp, DisplayVersion};
        e.print().unwrap_or_else(|e| eprintln!("{e}"));
        // return code 0 if we’re displaying help or version
        PySystemExit::new_err(!matches!(e.kind(), DisplayHelp | DisplayVersion))
    })?;
    let collector = collector::Collector::default();
    Python::with_gil(|py| {
        Bound::new(py, collector)?.register()?;

        let runpy = PyModule::import_bound(py, "runpy")?;
        let run_fn = if args.module { "run_module" } else { "run_path" };
        runpy.getattr(run_fn)?.call((args.name,), None)?;
        PyResult::Ok(())
    })?;
    Ok(())
}

/// Calculate and report region based code coverage.
#[pymodule]
fn fine_coverage(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
