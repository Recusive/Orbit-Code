//! Skills-specific watcher built on top of the generic [`FileWatcher`].

use std::path::PathBuf;
use std::sync::Arc;

use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tracing::warn;

use crate::SkillsManager;
use crate::config::Config;
use crate::file_watcher::FileWatcher;
use crate::file_watcher::FileWatcherEvent;
use crate::file_watcher::WatchRegistration;
use crate::plugins::PluginsManager;
use crate::skills_load_input_from_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillsWatcherEvent {
    SkillsChanged { paths: Vec<PathBuf> },
}

pub(crate) struct SkillsWatcher {
    file_watcher: Arc<FileWatcher>,
    tx: broadcast::Sender<SkillsWatcherEvent>,
}

impl SkillsWatcher {
    pub(crate) fn new(file_watcher: &Arc<FileWatcher>) -> Self {
        let rx = file_watcher.subscribe();
        let (tx, _) = broadcast::channel(128);
        let skills_watcher = Self {
            file_watcher: Arc::clone(file_watcher),
            tx: tx.clone(),
        };
        Self::spawn_event_loop(rx, tx);
        skills_watcher
    }

    pub(crate) fn noop() -> Self {
        Self::new(&Arc::new(FileWatcher::noop()))
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<SkillsWatcherEvent> {
        self.tx.subscribe()
    }

    pub(crate) fn register_config(
        &self,
        config: &Config,
        skills_manager: &SkillsManager,
        plugins_manager: &PluginsManager,
    ) -> WatchRegistration {
        let plugin_outcome = plugins_manager.plugins_for_config(config);
        let effective_skill_roots = plugin_outcome.effective_skill_roots();
        let skills_input = skills_load_input_from_config(config, effective_skill_roots);
        self.file_watcher
            .register_config(&skills_input, skills_manager)
    }

    fn spawn_event_loop(
        mut rx: broadcast::Receiver<FileWatcherEvent>,
        tx: broadcast::Sender<SkillsWatcherEvent>,
    ) {
        if let Ok(handle) = Handle::try_current() {
            handle.spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(FileWatcherEvent::SkillsChanged { paths }) => {
                            let _ = tx.send(SkillsWatcherEvent::SkillsChanged { paths });
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    }
                }
            });
        } else {
            warn!("skills watcher listener skipped: no Tokio runtime available");
        }
    }
}
