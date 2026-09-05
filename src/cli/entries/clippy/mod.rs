use std::sync::Arc;
use std::sync::LazyLock;

use clap::Parser;
use futures::SinkExt;
use futures::Stream;
use futures::StreamExt;
use futures::channel::mpsc::Sender;
use futures::join;
use futures::pin_mut;
use rand::seq::IndexedRandom;
use streamies::TryStreamies;
use tagstudio_db::Entry;
use tagstudio_db::models::library::Library;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

use crate::cli::entries::clippy::lints::ask_lint;
use crate::models::cli_utils::cli_data::CLI_DATA;

pub mod lint_config;
pub mod lints;

static PRINT_LOCK: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

/// Rename a tag, and add its previous name as alias
#[derive(Parser, Debug, Clone)]
pub struct EntriesClippyCommand {}

impl EntriesClippyCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let lib = Arc::new(CLI_DATA.read().await.get_library().await?);

        let entries = Entry::stream_entries(&mut *lib.db.get().await?)
            .try_collect_vec()
            .await?;
        let entries = Arc::new(RwLock::new(entries));

        let (task_send, task_recv) = futures::channel::mpsc::channel::<Entry>(5);

        let task_1 = create_tasks_task(entries.clone(), task_send);
        join! {
            task_1,
            check_lints_task(task_recv, lib, entries)
        };

        Ok(())
    }
}

async fn create_tasks_task(entries: Arc<RwLock<Vec<Entry>>>, mut sender: Sender<Entry>) {
    let mut rng = rand::rng();

    loop {
        let lock = entries.read().await;
        let Some(entry) = lock.choose(&mut rng).cloned() else {
            break;
        };
        drop(lock);
        sender.send(entry).await.unwrap();
    }
}

async fn check_lints_task(
    stream: impl Stream<Item = Entry>,
    lib: Arc<Library>,
    entries: Arc<RwLock<Vec<Entry>>>,
) {
    let stream = stream
        .map(|entry| check_lint(&lib, &entries, entry))
        .buffer_unordered(32);

    pin_mut!(stream);

    while let Some(_val) = stream.next().await {}
}

async fn check_lint(lib: &Arc<Library>, entries: &Arc<RwLock<Vec<Entry>>>, entry: Entry) {
    if !ask_lint(lib, &entry).await.unwrap() {
        remove_entry(entries, &entry).await;
    }
}

async fn remove_entry(entries: &Arc<RwLock<Vec<Entry>>>, entry: &Entry) {
    let mut entries = entries.write().await;
    entries.retain(|entr| entr.id != entry.id);

    if let Ok(lock) = PRINT_LOCK.try_acquire() {
        println!("{} entries remaining", entries.len());
        drop(lock);
    }
}
