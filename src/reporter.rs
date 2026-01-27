use std::collections::HashMap;

use ouroboros::self_referencing;

use crate::collector::Collector;

#[self_referencing]
pub struct FileStats {
    ast: ruff_python_ast::ModModule,
    #[borrows(ast)]
    #[not_covariant]
    stats: HashMap<ruff_python_ast::AnyNodeRef<'this>, usize>,
}

pub(crate) fn report(collector: &Collector) {
    for (filename, stats) in &collector.stats {
        eprintln!("{filename}:");
        for (node, count) in stats {
            eprintln!("  {node:?}: {count}");
        }
    }
}
