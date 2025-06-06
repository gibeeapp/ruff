use ruff_python_ast::{self as ast, Expr, Stmt};

use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;
use crate::fix;
use crate::{AlwaysFixableViolation, Fix};

/// ## What it does
/// Checks for the use of `__metaclass__ = type` in class definitions.
///
/// ## Why is this bad?
/// Since Python 3, `__metaclass__ = type` is implied and can thus be omitted.
///
/// ## Example
///
/// ```python
/// class Foo:
///     __metaclass__ = type
/// ```
///
/// Use instead:
///
/// ```python
/// class Foo: ...
/// ```
///
/// ## References
/// - [PEP 3115 – Metaclasses in Python 3000](https://peps.python.org/pep-3115/)
#[derive(ViolationMetadata)]
pub(crate) struct UselessMetaclassType;

impl AlwaysFixableViolation for UselessMetaclassType {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`__metaclass__ = type` is implied".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove `__metaclass__ = type`".to_string()
    }
}

/// UP001
pub(crate) fn useless_metaclass_type(
    checker: &Checker,
    stmt: &Stmt,
    value: &Expr,
    targets: &[Expr],
) {
    let [Expr::Name(ast::ExprName { id, .. })] = targets else {
        return;
    };
    if id != "__metaclass__" {
        return;
    }
    let Expr::Name(ast::ExprName { id, .. }) = value else {
        return;
    };
    if id != "type" {
        return;
    }

    let mut diagnostic = checker.report_diagnostic(UselessMetaclassType, stmt.range());
    let stmt = checker.semantic().current_statement();
    let parent = checker.semantic().current_statement_parent();
    let edit = fix::edits::delete_stmt(stmt, parent, checker.locator(), checker.indexer());
    diagnostic.set_fix(Fix::safe_edit(edit).isolate(Checker::isolation(
        checker.semantic().current_statement_parent_id(),
    )));
}
