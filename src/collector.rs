use std::collections::HashMap;

use ouroboros::self_referencing;
use pyo3::prelude::*;

use pyo3::types::PyFrame;

use crate::tracer::{TraceEvent, Tracer};

#[pyclass(unsendable)]
#[derive(Default)]
pub struct Collector {
    pub stats: HashMap<String, FileStats>,
}

#[self_referencing]
pub struct FileStats {
    ast: ruff_python_ast::ModModule,
    #[borrows(ast)]
    #[not_covariant]
    stats: HashMap<ruff_python_ast::AnyNodeRef<'this>, usize>,
}

impl Tracer<TraceEvent> for Collector {
    fn trace(&mut self, frame: Py<PyFrame>, event: TraceEvent, py: Python) -> PyResult<()> {
        let frame = frame.into_bound(py);
        /*
        match event {
            TraceEvent::Exception {
                ..exc_type,
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
        */
        if !matches!(event, TraceEvent::Line) {
            return Ok(());
        }
        let code = frame.getattr("f_code")?;
        let positions = code.getattr("co_positions")?.call0()?;
        let filename = code.getattr("co_filename")?;
        for pos in positions.iter()? {
            let pos = pos?;
            if let Ok((sl, el, sc, ec)) = pos.extract::<(u32, u32, u32, u32)>() {
                dbg!((sl, el, sc, ec), &filename);
            }
        }
        Ok(())
    }
}
