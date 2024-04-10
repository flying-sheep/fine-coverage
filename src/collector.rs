use std::collections::HashMap;

use pyo3::prelude::*;

use pyo3::types::PyFrame;

use crate::tracer::{TraceEvent, Tracer};

type LineStats = HashMap<(u32, u32, u32, u32), usize>;

#[pyclass(unsendable)]
#[derive(Default)]
pub struct Collector {
    pub stats: HashMap<String, LineStats>,
}

impl Tracer<TraceEvent> for Collector {
    fn trace(&mut self, frame: Py<PyFrame>, event: TraceEvent, py: Python) -> PyResult<()> {
        let frame = frame.into_bound(py);
        if !matches!(event, TraceEvent::Line) {
            return Ok(());
        }
        let code = frame.getattr("f_code")?;
        let positions = code.getattr("co_positions")?.call0()?;
        let filename = code.getattr("co_filename")?;
        let filename = filename.extract::<&str>()?;
        if filename.starts_with('<') {
            // TODO: only collect own file instead
            return Ok(());
        }

        let stats = self.stats.entry(filename.to_owned()).or_default();
        for pos in positions
            .iter()?
            .filter_map(Result::ok)
            .filter_map(|pos| pos.extract::<(u32, u32, u32, u32)>().ok())
        {
            *stats.entry(pos).or_default() += 1;
        }
        Ok(())
    }
}
