use rustc_hash::FxHashMap;

use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_semantic::{Binding, Imported};
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;
use crate::{Fix, FixAvailability, Violation};

use crate::renamer::Renamer;

/// ## What it does
/// Checks for imports that are typically imported using a common convention,
/// like `import pandas as pd`, and enforces that convention.
///
/// ## Why is this bad?
/// Consistency is good. Use a common convention for imports to make your code
/// more readable and idiomatic.
///
/// For example, `import pandas as pd` is a common
/// convention for importing the `pandas` library, and users typically expect
/// Pandas to be aliased as `pd`.
///
/// ## Example
/// ```python
/// import pandas
/// ```
///
/// Use instead:
/// ```python
/// import pandas as pd
/// ```
///
/// ## Options
/// - `lint.flake8-import-conventions.aliases`
/// - `lint.flake8-import-conventions.extend-aliases`
#[derive(ViolationMetadata)]
pub(crate) struct UnconventionalImportAlias {
    name: String,
    asname: String,
}

impl Violation for UnconventionalImportAlias {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let UnconventionalImportAlias { name, asname } = self;
        format!("`{name}` should be imported as `{asname}`")
    }

    fn fix_title(&self) -> Option<String> {
        let UnconventionalImportAlias { name, asname } = self;
        Some(format!("Alias `{name}` to `{asname}`"))
    }
}

/// ICN001
pub(crate) fn unconventional_import_alias(
    checker: &Checker,
    binding: &Binding,
    conventions: &FxHashMap<String, String>,
) {
    let Some(import) = binding.as_any_import() else {
        return;
    };
    let qualified_name = import.qualified_name().to_string();
    let Some(expected_alias) = conventions.get(qualified_name.as_str()) else {
        return;
    };

    let name = binding.name(checker.source());
    if name == expected_alias {
        return;
    }

    let mut diagnostic = checker.report_diagnostic(
        UnconventionalImportAlias {
            name: qualified_name,
            asname: expected_alias.to_string(),
        },
        binding.range(),
    );
    if !import.is_submodule_import() {
        if checker.semantic().is_available(expected_alias) {
            diagnostic.try_set_fix(|| {
                let scope = &checker.semantic().scopes[binding.scope];
                let (edit, rest) = Renamer::rename(
                    name,
                    expected_alias,
                    scope,
                    checker.semantic(),
                    checker.stylist(),
                )?;
                Ok(Fix::unsafe_edits(edit, rest))
            });
        }
    }
}
