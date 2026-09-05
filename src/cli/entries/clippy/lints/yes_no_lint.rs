use std::sync::LazyLock;

use inquire_derive::Selectable;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tagstudio_db::Tag;

use crate::ColEyre;

use crate::cli::entries::clippy::PRINT_LOCK;
use crate::cli::entries::clippy::lint_config::YesNoLint;
use crate::cli::entries::clippy::lints::EntryLint;
use crate::cli::entries::clippy::lints::LintAction;
use crate::interface::images::cache::IMAGE_CACHE;
use crate::interface::images::print_entry_to_cli;

static DISPLAY_CONF: LazyLock<viuer::Config> = LazyLock::new(|| viuer::Config {
    height: Some(55),
    x: 0,
    y: 0,
    allow_vscode: true,
    ..Default::default()
});

#[derive(Debug, Clone, Copy, Selectable, strum::Display)]
enum Response {
    Yes,
    No,
    Skip,
}

impl YesNoLint {
    /// Check if lint should be skipped
    pub async fn skip(&self, lib: &Library, entry: &Entry) -> ColEyre<bool> {
        let mut blacklist = self.tag_blacklist_any.clone();
        blacklist.extend(self.add_yes_tags.clone());
        blacklist.extend(self.add_no_tags.clone());

        Ok(Self::entry_has_tag_in(lib, entry, &blacklist).await?
            || (!self.tag_whitelist_any.is_empty()
                && !Self::entry_has_tag_in(lib, entry, &self.tag_whitelist_any).await?))
    }

    pub async fn entry_has_tag_in(
        lib: &Library,
        entry: &Entry,
        tags_list: &Vec<String>,
    ) -> ColEyre<bool> {
        let entry_tags = entry.get_tags(&mut *lib.db.get().await?).await?;

        for skip_tag in tags_list {
            let skip_tags =
                Tag::get_by_name_or_insert_new(&mut *lib.db.get().await?, skip_tag.to_string())
                    .await?;
            for skip_tag in skip_tags {
                if entry_tags.iter().any(|etag| etag.id == skip_tag.id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub async fn add_tags(&self, lib: &Library, entry: &Entry, tags: &Vec<String>) -> ColEyre {
        for tag in tags {
            let db_tags =
                Tag::get_by_name_or_insert_new(&mut *lib.db.get().await?, tag.to_string()).await?;
            entry.add_tags(&mut *lib.db.get().await?, &db_tags).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl EntryLint for YesNoLint {
    async fn check_lint(&self, lib: &Library, entry: &Entry) -> ColEyre<LintAction> {
        if self.skip(lib, entry).await? {
            return Ok(LintAction::NotApplicable);
        }

        IMAGE_CACHE.get_or_init(lib, entry.id).await?;
        let plock = PRINT_LOCK.acquire().await.unwrap();
        print!("{}[2J", 27 as char);
        print_entry_to_cli(lib, entry, &DISPLAY_CONF).await?;
        println!("path: {}", entry.path);

        let question = self.question.clone();
        let res = tokio::spawn(async move { Response::select(&question).prompt() }).await??;
        drop(plock);

        match res {
            Response::Skip => return Ok(LintAction::Skiped),
            Response::Yes => {
                self.add_tags(lib, entry, &self.add_yes_tags).await?;
            }
            Response::No => {
                self.add_tags(lib, entry, &self.add_no_tags).await?;
            }
        }

        Ok(LintAction::Applied)
    }
}
