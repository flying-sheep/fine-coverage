use std::collections::HashMap;

use ouroboros::self_referencing;

#[self_referencing]
pub struct FileStats {
    ast: ruff_python_ast::ModModule,
    #[borrows(ast)]
    #[not_covariant]
    stats: HashMap<ruff_python_ast::AnyNodeRef<'this>, usize>,
}
