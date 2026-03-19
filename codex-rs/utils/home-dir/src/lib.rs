use dirs::home_dir;
use std::io;
use std::path::Path;
use std::path::PathBuf;

const ORBIT_HOME_ENV_VAR: &str = "ORBIT_HOME";
const LEGACY_CODEX_HOME_ENV_VAR: &str = "CODEX_HOME";
const ORBIT_HOME_DIR_NAME: &str = ".orbit";
const LEGACY_CODEX_HOME_DIR_NAME: &str = ".codex";

/// Returns the path to the Orbit Code configuration directory.
///
/// Resolution order:
/// 1. `ORBIT_HOME`
/// 2. `CODEX_HOME` (legacy)
/// 3. `~/.orbit` when it already exists
/// 4. `~/.codex` when it already exists
/// 5. `~/.orbit` for fresh installs
///
/// When an environment variable is set, the value must exist and be a
/// directory. Existing on-disk defaults are not canonicalized so callers can
/// decide whether they want to require the directory to exist yet.
pub fn find_orbit_home() -> io::Result<PathBuf> {
    let orbit_home_env = std::env::var(ORBIT_HOME_ENV_VAR)
        .ok()
        .filter(|value| !value.is_empty());
    let legacy_codex_home_env = std::env::var(LEGACY_CODEX_HOME_ENV_VAR)
        .ok()
        .filter(|value| !value.is_empty());
    find_orbit_home_from_envs(
        orbit_home_env.as_deref(),
        legacy_codex_home_env.as_deref(),
        home_dir(),
    )
}

pub fn find_orbit_code_home() -> io::Result<PathBuf> {
    find_orbit_home()
}

pub fn find_codex_home() -> io::Result<PathBuf> {
    find_orbit_home()
}

fn find_orbit_home_from_envs(
    orbit_home_env: Option<&str>,
    legacy_codex_home_env: Option<&str>,
    user_home: Option<PathBuf>,
) -> io::Result<PathBuf> {
    if let Some(value) = orbit_home_env {
        return validate_home_env_var(ORBIT_HOME_ENV_VAR, value);
    }
    if let Some(value) = legacy_codex_home_env {
        return validate_home_env_var(LEGACY_CODEX_HOME_ENV_VAR, value);
    }

    let user_home = user_home
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find home directory"))?;
    let orbit_home = user_home.join(ORBIT_HOME_DIR_NAME);
    if existing_directory(&orbit_home) {
        return Ok(orbit_home);
    }

    let legacy_codex_home = user_home.join(LEGACY_CODEX_HOME_DIR_NAME);
    if existing_directory(&legacy_codex_home) {
        return Ok(legacy_codex_home);
    }

    Ok(orbit_home)
}

fn validate_home_env_var(env_var_name: &str, value: &str) -> io::Result<PathBuf> {
    let path = PathBuf::from(value);
    let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => io::Error::new(
            io::ErrorKind::NotFound,
            format!("{env_var_name} points to {value:?}, but that path does not exist"),
        ),
        _ => io::Error::new(
            err.kind(),
            format!("failed to read {env_var_name} {value:?}: {err}"),
        ),
    })?;

    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{env_var_name} points to {value:?}, but that path is not a directory"),
        ));
    }

    path.canonicalize().map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("failed to canonicalize {env_var_name} {value:?}: {err}"),
        )
    })
}

fn existing_directory(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::find_orbit_home_from_envs;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_orbit_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err = find_orbit_home_from_envs(Some(missing_str), None, None)
            .expect_err("missing ORBIT_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("ORBIT_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_legacy_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err = find_orbit_home_from_envs(None, Some(missing_str), None)
            .expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_orbit_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err =
            find_orbit_home_from_envs(Some(file_str), None, None).expect_err("file ORBIT_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_orbit_home_env_takes_precedence_over_legacy_codex_home() {
        let orbit_home = TempDir::new().expect("orbit temp home");
        let legacy_codex_home = TempDir::new().expect("legacy temp home");
        let orbit_home_str = orbit_home
            .path()
            .to_str()
            .expect("orbit home path should be valid utf-8");
        let legacy_codex_home_str = legacy_codex_home
            .path()
            .to_str()
            .expect("legacy codex home path should be valid utf-8");

        let resolved =
            find_orbit_home_from_envs(Some(orbit_home_str), Some(legacy_codex_home_str), None)
                .expect("ORBIT_HOME should win");
        let expected = orbit_home
            .path()
            .canonicalize()
            .expect("canonicalize orbit home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_orbit_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved =
            find_orbit_home_from_envs(Some(temp_str), None, None).expect("valid ORBIT_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_orbit_home_without_env_prefers_existing_orbit_dir() {
        let temp_home = TempDir::new().expect("temp home");
        let expected = temp_home.path().join(".orbit");
        fs::create_dir_all(&expected).expect("create .orbit");

        let resolved = find_orbit_home_from_envs(None, None, Some(temp_home.path().to_path_buf()))
            .expect("default ORBIT_HOME");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_orbit_home_without_env_falls_back_to_existing_legacy_codex_dir() {
        let temp_home = TempDir::new().expect("temp home");
        let expected = temp_home.path().join(".codex");
        fs::create_dir_all(&expected).expect("create .codex");

        let resolved = find_orbit_home_from_envs(None, None, Some(temp_home.path().to_path_buf()))
            .expect("legacy CODEX_HOME fallback");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_orbit_home_without_env_defaults_to_orbit_dir_for_fresh_install() {
        let temp_home = TempDir::new().expect("temp home");
        let expected = temp_home.path().join(".orbit");

        let resolved = find_orbit_home_from_envs(None, None, Some(temp_home.path().to_path_buf()))
            .expect("fresh install default");
        assert_eq!(resolved, expected);
    }
}
