use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;
use crate::cli::entries::clippy::lint_config::EntryLintConfig;
use crate::cli::entries::clippy::lint_config::LintsConfig;
use crate::cli::entries::clippy::lint_config::delegate_entry_lint_config;

pub mod yes_no_lint;

enum LintAction {
    NotApplicable,
    Skiped,
    Applied,
}

#[async_trait::async_trait]
trait EntryLint {
    async fn check_lint(&self, lib: &Library, entry: &Entry) -> ColEyre<LintAction>;
}

pub(super) async fn ask_lint(lib: &Library, entry: &Entry) -> ColEyre<bool> {
    let lints = LintsConfig::load(lib).unwrap();

    for lint in lints.lint {
        let res = delegate_entry_lint_config! {lint.check_lint(lib, entry).await?};

        match res {
            LintAction::Applied => return Ok(true),
            LintAction::Skiped => return Ok(true),
            LintAction::NotApplicable => {}
        }
    }

    Ok(false)
}
