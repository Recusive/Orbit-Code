use std::path::Path;

use crate::config::Config;
use crate::error::CodexErr;
use orbit_code_protocol::protocol::RolloutItem;

pub use orbit_code_rollout::ARCHIVED_SESSIONS_SUBDIR;
pub use orbit_code_rollout::INTERACTIVE_SESSION_SOURCES;
pub use orbit_code_rollout::RolloutRecorder;
pub use orbit_code_rollout::RolloutRecorderParams;
pub use orbit_code_rollout::SESSIONS_SUBDIR;
pub use orbit_code_rollout::SessionMeta;
pub use orbit_code_rollout::append_thread_name;
pub use orbit_code_rollout::find_archived_thread_path_by_id_str;
#[deprecated(note = "use find_thread_path_by_id_str")]
pub use orbit_code_rollout::find_conversation_path_by_id_str;
pub use orbit_code_rollout::find_thread_name_by_id;
pub use orbit_code_rollout::find_thread_path_by_id_str;
pub use orbit_code_rollout::find_thread_path_by_name_str;
pub use orbit_code_rollout::rollout_date_parts;

impl orbit_code_rollout::RolloutConfigView for Config {
    fn orbit_code_home(&self) -> &std::path::Path {
        self.orbit_code_home.as_path()
    }

    fn sqlite_home(&self) -> &std::path::Path {
        self.sqlite_home.as_path()
    }

    fn cwd(&self) -> &std::path::Path {
        self.cwd.as_path()
    }

    fn model_provider_id(&self) -> &str {
        self.model_provider_id.as_str()
    }

    fn generate_memories(&self) -> bool {
        self.memories.generate_memories
    }
}

pub mod list {
    pub use orbit_code_rollout::list::*;
}

pub(crate) mod metadata {
    pub(crate) use orbit_code_rollout::metadata::backfill_sessions;
    pub(crate) use orbit_code_rollout::metadata::builder_from_items;
    pub(crate) use orbit_code_rollout::metadata::extract_metadata_from_rollout;
}

pub mod policy {
    pub use orbit_code_rollout::policy::*;
}

pub mod recorder {
    pub use orbit_code_rollout::recorder::*;
}

pub mod session_index {
    pub use orbit_code_rollout::session_index::*;
}

pub(crate) fn map_session_init_error(err: &anyhow::Error, orbit_code_home: &Path) -> CodexErr {
    if let Some(mapped) = err
        .chain()
        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
        .find_map(|io_err| map_rollout_io_error(io_err, orbit_code_home))
    {
        return mapped;
    }

    CodexErr::Fatal(format!("Failed to initialize session: {err:#}"))
}

fn map_rollout_io_error(io_err: &std::io::Error, orbit_code_home: &Path) -> Option<CodexErr> {
    let sessions_dir = orbit_code_home.join(SESSIONS_SUBDIR);
    let hint = match io_err.kind() {
        std::io::ErrorKind::PermissionDenied => format!(
            "Codex cannot access session files at {} (permission denied). If sessions were created using sudo, fix ownership: sudo chown -R $(whoami) {}",
            sessions_dir.display(),
            orbit_code_home.display()
        ),
        std::io::ErrorKind::NotFound => format!(
            "Session storage missing at {}. Create the directory or choose a different Codex home.",
            sessions_dir.display()
        ),
        std::io::ErrorKind::AlreadyExists => format!(
            "Session storage path {} is blocked by an existing file. Remove or rename it so Codex can create sessions.",
            sessions_dir.display()
        ),
        std::io::ErrorKind::InvalidData | std::io::ErrorKind::InvalidInput => format!(
            "Session data under {} looks corrupt or unreadable. Clearing the sessions directory may help (this will remove saved threads).",
            sessions_dir.display()
        ),
        std::io::ErrorKind::IsADirectory | std::io::ErrorKind::NotADirectory => format!(
            "Session storage path {} has an unexpected type. Ensure it is a directory Codex can use for session files.",
            sessions_dir.display()
        ),
        _ => return None,
    };

    Some(CodexErr::Fatal(format!(
        "{hint} (underlying error: {io_err})"
    )))
}

pub(crate) mod truncation {
    use super::RolloutItem;
    use crate::event_mapping;
    use orbit_code_protocol::items::TurnItem;
    use orbit_code_protocol::models::ResponseItem;
    use orbit_code_protocol::protocol::EventMsg;

    pub(crate) fn user_message_positions_in_rollout(items: &[RolloutItem]) -> Vec<usize> {
        let mut user_positions = Vec::new();
        for (idx, item) in items.iter().enumerate() {
            match item {
                RolloutItem::ResponseItem(item @ ResponseItem::Message { .. })
                    if matches!(
                        event_mapping::parse_turn_item(item),
                        Some(TurnItem::UserMessage(_))
                    ) =>
                {
                    user_positions.push(idx);
                }
                RolloutItem::EventMsg(EventMsg::ThreadRolledBack(rollback)) => {
                    let num_turns = usize::try_from(rollback.num_turns).unwrap_or(usize::MAX);
                    let new_len = user_positions.len().saturating_sub(num_turns);
                    user_positions.truncate(new_len);
                }
                _ => {}
            }
        }
        user_positions
    }

    pub(crate) fn truncate_rollout_before_nth_user_message_from_start(
        items: &[RolloutItem],
        n_from_start: usize,
    ) -> Vec<RolloutItem> {
        if n_from_start == usize::MAX {
            return items.to_vec();
        }

        let user_positions = user_message_positions_in_rollout(items);
        if user_positions.len() <= n_from_start {
            return Vec::new();
        }

        let cut_idx = user_positions[n_from_start];
        items[..cut_idx].to_vec()
    }
}
