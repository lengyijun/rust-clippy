use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::source::snippet_opt;
use rustc_ast::{Item, ItemKind, ast};
use rustc_data_structures::fx::FxHashSet;
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::impl_lint_pass;
use rustc_span::FileName;
use rustc_span::def_id::LOCAL_CRATE;
use std::ffi::OsString;
use std::path::Path;

declare_clippy_lint! {
    /// ### What it does
    /// This lint checks for imports from the current crate that do not use the `crate::` prefix.
    /// It suggests using `crate::` to make it clear that the item is from the same crate.
    ///
    /// ### Why is this bad?
    /// When imports from the current crate lack the `crate::` prefix, it can make the code less readable
    /// because it’s not immediately clear if the imported item is from the current crate or an external dependency.
    /// Using `crate::` for self-imports provides a consistent style, making the origin of each import clear.
    /// This helps reduce confusion and maintain a uniform codebase.
    ///
    /// ### Example
    /// ```no_run
    /// use foo::bar; // foo is a module in the current crate
    /// ```
    /// Use instead:
    /// ```no_run
    /// use crate::foo::bar;
    /// ```
    #[clippy::version = "1.84.0"]
    pub USE_CRATE_PREFIX_FOR_SELF_IMPORTS,
    style,
    "checks that imports from the current crate use the `crate::` prefix"
}

impl_lint_pass!(UseCratePrefixForSelfImports => [USE_CRATE_PREFIX_FOR_SELF_IMPORTS]);

#[derive(Default)]
pub struct UseCratePrefixForSelfImports {
    mod_set: FxHashSet<OsString>,
}

impl EarlyLintPass for UseCratePrefixForSelfImports {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _: &ast::Crate) {
        let files = cx.sess().source_map().files();

        let Some(trim_to_src) = cx.sess().opts.working_dir.local_path() else {
            return;
        };

        for file in files.iter() {
            if let FileName::Real(name) = &file.name
                && let Some(lp) = name.local_path()
                && file.cnum == LOCAL_CRATE
            {
                let path = if lp.is_relative() {
                    lp
                } else if let Ok(relative) = lp.strip_prefix(trim_to_src) {
                    relative
                } else {
                    continue;
                };

                if let Some(root) = path.components().nth(1) {
                    let root: &Path = root.as_ref();
                    if let Some(mod_name) = root.file_stem() {
                        self.mod_set.insert(mod_name.to_owned());
                    };
                }
            }
        }
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Use(use_tree) = &item.kind {
            if let Some(x) = use_tree.prefix.segments.first() {
                if self.mod_set.contains(&OsString::from(x.ident.name.as_str())) {
                    span_lint_and_sugg(
                        cx,
                        USE_CRATE_PREFIX_FOR_SELF_IMPORTS,
                        use_tree.span,
                        "this import is not clear",
                        "prefix with `crate::`",
                        format!("crate::{}", snippet_opt(cx, use_tree.span).unwrap()),
                        Applicability::MachineApplicable,
                    );
                }
            }
        }
    }
}
