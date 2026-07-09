#![warn(clippy::pedantic)]

mod collector;
mod reporter;
mod tracer;

use clap::Parser;
use pyo3::exceptions::{PySystemExit, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use regex::Regex;

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
    let filter = args
        .cov
        .as_deref()
        .map_or_else(
            || Regex::new("^(|[^<].*)$"),
            |cov| Regex::new(&format!(r"^(.+/)?{}(/.+)?\.py$", regex::escape(cov))),
        )
        .map_err(|e| PyValueError::new_err(format!("Could not regex: {e}")))?;
    let collector: collector::Collector = Python::with_gil(|py| {
        let collector = Bound::new(py, collector::Collector::new(filter))?;
        collector.clone().register()?;
        runpy(args, py)?;
        collector.clone().deregister()?;
        collector.extract()
    })?;
    reporter::report(&collector);
    Ok(())
}

fn runpy(args: Args, py: Python) -> PyResult<()> {
    let sys = PyModule::import_bound(py, "sys")?;
    let runpy = PyModule::import_bound(py, "runpy")?;
    // prepare runpy function and arguments
    let kwargs = PyDict::new_bound(py);
    kwargs.set_item("run_name", "__main__")?;
    let run_fn = runpy.getattr(if args.module {
        // alter sys.argv[0] and `sys.modules["__main__"]`
        kwargs.set_item("alter_sys", true)?;
        "run_module"
    } else {
        "run_path"
    })?;
    // prepare sys.argv
    let sys_argv = sys.getattr("argv")?.extract::<Vec<String>>()?;
    sys.setattr("argv", {
        // sys.argv[0] will be reset by runpy
        let mut sys_argv = vec![String::new()];
        sys_argv.extend(args.options);
        sys_argv
    })?;
    // call runpy function and restore sys.argv
    let res = run_fn.call((args.name,), Some(&kwargs));
    sys.setattr("argv", sys_argv)?;
    res?;
    Ok(())
}

/// Calculate and report region based code coverage.
#[pymodule]
fn fine_coverage(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
