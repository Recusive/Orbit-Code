use dirs::home_dir;
use std::io;
use std::path::PathBuf;

const ORBIT_HOME_ENV_VAR: &str = "ORBIT_HOME";
const ORBIT_HOME_DIR_NAME: &str = ".orbit";

/// Returns the path to the Orbit Code configuration directory.
///
/// Resolution order:
/// 1. `ORBIT_HOME`
/// 2. `~/.orbit`
///
/// When an environment variable is set, the value must exist and be a
/// directory. Existing on-disk defaults are not canonicalized so callers can
/// decide whether they want to require the directory to exist yet.
pub fn find_orbit_home() -> io::Result<PathBuf> {
    let orbit_home_env = std::env::var(ORBIT_HOME_ENV_VAR)
        .ok()
        .filter(|value| !value.is_empty());
    find_orbit_home_from_envs(orbit_home_env.as_deref(), home_dir())
}

pub fn find_orbit_code_home() -> io::Result<PathBuf> {
    find_orbit_home()
}

fn find_orbit_home_from_envs(
    orbit_home_env: Option<&str>,
    user_home: Option<PathBuf>,
) -> io::Result<PathBuf> {
    if let Some(value) = orbit_home_env {
        return validate_home_env_var(ORBIT_HOME_ENV_VAR, value);
    }

    let user_home = user_home
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find home directory"))?;
    Ok(user_home.join(ORBIT_HOME_DIR_NAME))
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

        let err =
            find_orbit_home_from_envs(Some(missing_str), None).expect_err("missing ORBIT_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("ORBIT_HOME"),
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

        let err = find_orbit_home_from_envs(Some(file_str), None).expect_err("file ORBIT_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_orbit_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_orbit_home_from_envs(Some(temp_str), None).expect("valid ORBIT_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_orbit_home_without_env_defaults_to_orbit_dir() {
        let temp_home = TempDir::new().expect("temp home");
        let expected = temp_home.path().join(".orbit");

        let resolved = find_orbit_home_from_envs(None, Some(temp_home.path().to_path_buf()))
            .expect("default ORBIT_HOME");
        assert_eq!(resolved, expected);
    }
}
