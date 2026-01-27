use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyFrame;
use regex::Regex;

use crate::tracer::{TraceEvent, Tracer};

type LineStats = HashMap<(u32, u32, u32, u32), usize>;

#[pyclass]
#[derive(Clone)]
pub struct Collector {
    pub filter: Regex,
    pub stats: HashMap<String, LineStats>,
}

impl Collector {
    pub fn new(filter: Regex) -> Self {
        Self {
            filter,
            stats: HashMap::new(),
        }
    }
}

impl Tracer<TraceEvent> for Collector {
    fn trace(&mut self, frame: Py<PyFrame>, event: TraceEvent, py: Python) -> PyResult<()> {
        let frame = frame.into_bound(py);
        if !matches!(event, TraceEvent::Line) {
            return Ok(());
        }
        let code = frame.getattr("f_code")?;
        let filename = code.getattr("co_filename")?;
        let filename = filename.extract::<&str>()?;
        // TODO: more precise filtering
        if !self.filter.is_match(filename) {
            return Ok(());
        }

        let stats = self.stats.entry(filename.to_owned()).or_default();
        for pos in code
            .getattr("co_positions")?
            .call0()?
            .iter()?
            .filter_map(Result::ok)
            .filter_map(|pos| pos.extract::<(u32, u32, u32, u32)>().ok())
        {
            *stats.entry(pos).or_default() += 1;
        }
        Ok(())
    }
}
