pub(crate) use orbit_code_skills::install_system_skills;
pub(crate) use orbit_code_skills::system_cache_root_dir;

use std::path::Path;

pub(crate) fn uninstall_system_skills(orbit_code_home: &Path) {
    let system_skills_dir = system_cache_root_dir(orbit_code_home);
    let _ = std::fs::remove_dir_all(&system_skills_dir);
}
