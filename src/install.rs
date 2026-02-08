use std::path::{Path, PathBuf};

pub struct EnvSnapshot<'a> {
    pub home: Option<&'a Path>,
    pub xdg_bin_home: Option<&'a Path>,
    pub path: Option<&'a str>,
}

pub fn install_dir(env: EnvSnapshot<'_>) -> Option<PathBuf> {
    if let Some(xdg) = env.xdg_bin_home {
        return Some(xdg.to_path_buf());
    }

    let home = env.home?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_dir_prefers_xdg_bin_home() {
        let env = EnvSnapshot {
            home: Some(Path::new("/home/demo")),
            xdg_bin_home: Some(Path::new("/custom/bin")),
            path: None,
        };
        assert_eq!(install_dir(env), Some(PathBuf::from("/custom/bin")));
    }

    #[test]
    fn install_dir_falls_back_to_home_local_bin() {
        let env = EnvSnapshot {
            home: Some(Path::new("/home/demo")),
            xdg_bin_home: None,
            path: None,
        };
        assert_eq!(
            install_dir(env),
            Some(PathBuf::from("/home/demo/.local/bin"))
        );
    }

    #[test]
    fn install_dir_none_when_no_home() {
        let env = EnvSnapshot {
            home: None,
            xdg_bin_home: None,
            path: None,
        };
        assert_eq!(install_dir(env), None);
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
}
