use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn install_dir(env: &crate::EnvSnapshot) -> Option<PathBuf> {
    if let Some(xdg) = env.xdg_bin_home.as_deref() {
        return Some(xdg.to_path_buf());
    }

    let home = env.home.as_deref()?;
    Some(home.join(".local").join("bin"))
}

pub fn is_on_path(dir: &Path, path_var: Option<&str>) -> bool {
    let path_var = match path_var {
        Some(path_var) => path_var,
        None => return false,
    };

    std::env::split_paths(path_var).any(|entry| entry == dir)
}

pub fn render_wrapper(crate_root: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nREPO=\"{}\"\nexec cargo run --manifest-path \"$REPO/Cargo.toml\" -- \"$@\"\n",
        crate_root.display()
    )
}

pub fn write_wrapper(wrapper_path: &Path, contents: &str, force: bool) -> io::Result<()> {
    if wrapper_path.exists() && !force {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "wrapper already exists",
        ));
    }

    if let Some(parent) = wrapper_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = wrapper_path.with_extension("tmp");
    fs::write(&temp_path, contents)?;

    let mut perms = fs::metadata(&temp_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_path, perms)?;

    fs::rename(&temp_path, wrapper_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;

    #[test]
    fn install_dir_prefers_xdg_bin_home() {
        let env = crate::EnvSnapshot {
            home: Some(PathBuf::from("/home/demo")),
            xdg_bin_home: Some(PathBuf::from("/custom/bin")),
            path: None,
        };
        assert_eq!(install_dir(&env), Some(PathBuf::from("/custom/bin")));
    }

    #[test]
    fn install_dir_falls_back_to_home_local_bin() {
        let env = crate::EnvSnapshot {
            home: Some(PathBuf::from("/home/demo")),
            xdg_bin_home: None,
            path: None,
        };
        assert_eq!(
            install_dir(&env),
            Some(PathBuf::from("/home/demo/.local/bin"))
        );
    }

    #[test]
    fn install_dir_none_when_no_home() {
        let env = crate::EnvSnapshot {
            home: None,
            xdg_bin_home: None,
            path: None,
        };
        assert_eq!(install_dir(&env), None);
    }

    #[test]
    fn is_on_path_detects_match() {
        let dir = Path::new("/home/demo/.local/bin");
        let path_var = "/usr/bin:/home/demo/.local/bin:/bin";
        assert!(is_on_path(dir, Some(path_var)));
    }

    #[test]
    fn is_on_path_handles_missing_path() {
        let dir = Path::new("/home/demo/.local/bin");
        assert!(!is_on_path(dir, None));
    }

    #[test]
    fn is_on_path_returns_false_for_absent_dir() {
        let dir = Path::new("/opt/bin");
        let path_var = "/usr/bin:/home/demo/.local/bin:/bin";
        assert!(!is_on_path(dir, Some(path_var)));
    }

    #[test]
    fn render_wrapper_contains_expected_lines() {
        let wrapper = render_wrapper(Path::new("/repo/root"));
        assert!(wrapper.starts_with("#!/usr/bin/env bash\n"));
        assert!(wrapper.contains("set -euo pipefail\n"));
        assert!(wrapper.contains("REPO=\"/repo/root\"\n"));
        assert!(wrapper.contains("exec cargo run --manifest-path \"$REPO/Cargo.toml\" -- \"$@\"\n"));
    }

    #[test]
    fn render_wrapper_quotes_repo_paths_with_spaces() {
        let wrapper = render_wrapper(Path::new("/path with spaces/repo"));
        assert!(wrapper.contains("REPO=\"/path with spaces/repo\"\n"));
    }

    #[test]
    fn write_wrapper_creates_parent_and_sets_executable() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let wrapper_path = temp_dir.path().join("bin").join("demo");

        write_wrapper(&wrapper_path, "echo demo\n", false).expect("write wrapper");

        let metadata = fs::metadata(&wrapper_path).expect("wrapper metadata");
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o111, 0o111);
    }

    #[test]
    fn write_wrapper_refuses_overwrite_without_force() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let wrapper_path = temp_dir.path().join("demo");
        write_wrapper(&wrapper_path, "echo demo\n", false).expect("write wrapper");

        let err = write_wrapper(&wrapper_path, "echo other\n", false)
            .expect_err("expected already exists");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn write_wrapper_overwrites_with_force() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let wrapper_path = temp_dir.path().join("demo");
        write_wrapper(&wrapper_path, "echo demo\n", false).expect("write wrapper");

        write_wrapper(&wrapper_path, "echo other\n", true).expect("overwrite");
        let contents = fs::read_to_string(&wrapper_path).expect("read wrapper");
        assert_eq!(contents, "echo other\n");
    }

    #[test]
    fn write_wrapper_overwrites_existing_regular_file_with_force() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let wrapper_path = temp_dir.path().join("demo");
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&wrapper_path)
            .expect("create file");

        write_wrapper(&wrapper_path, "echo demo\n", true).expect("overwrite");
        let contents = fs::read_to_string(&wrapper_path).expect("read wrapper");
        assert_eq!(contents, "echo demo\n");
    }
}
