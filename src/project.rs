use cargo_metadata::{MetadataCommand, TargetKind};
use std::path::{Path, PathBuf};

pub fn find_crate_root(cwd: &Path) -> Result<PathBuf, String> {
    let mut current = if cwd.is_absolute() {
        cwd.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| format!("failed to resolve current dir: {err}"))?
            .join(cwd)
    };

    loop {
        let manifest = current.join("Cargo.toml");
        if manifest.is_file() {
            return Ok(current);
        }

        if !current.pop() {
            return Err("Cargo.toml not found in this directory or any parent".to_string());
        }
    }
}

pub fn list_bins(manifest_path: &Path) -> Result<Vec<String>, String> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
        .map_err(|err| format!("failed to load cargo metadata: {err}"))?;

    let root = metadata
        .root_package()
        .ok_or_else(|| "no root package found in Cargo metadata".to_string())?;

    Ok(root
        .targets
        .iter()
        .filter(|target| {
            target
                .kind
                .iter()
                .any(|kind| matches!(kind, TargetKind::Bin))
        })
        .map(|target| target.name.clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn find_crate_root_in_current_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        );

        let root = find_crate_root(dir.path()).expect("crate root");
        assert_eq!(root, dir.path());
    }

    #[test]
    fn find_crate_root_in_parent_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        );
        let nested = dir.path().join("nested/child");
        fs::create_dir_all(&nested).expect("create nested");

        let root = find_crate_root(&nested).expect("crate root");
        assert_eq!(root, dir.path());
    }

    #[test]
    fn find_crate_root_errors_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = find_crate_root(dir.path()).expect_err("expected error");
        assert!(err.contains("Cargo.toml not found"));
    }

    #[test]
    fn list_bins_returns_single_bin() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.rs\"\n",
        );
        write_file(&dir.path().join("src/main.rs"), "fn main() {}\n");

        let bins = list_bins(&dir.path().join("Cargo.toml")).expect("bins");
        assert_eq!(bins, vec!["demo".to_string()]);
    }

    #[test]
    fn list_bins_returns_multiple_bins() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"alpha\"\npath = \"src/main.rs\"\n\n[[bin]]\nname = \"beta\"\npath = \"src/bin/beta.rs\"\n",
        );
        write_file(&dir.path().join("src/main.rs"), "fn main() {}\n");
        write_file(&dir.path().join("src/bin/beta.rs"), "fn main() {}\n");

        let mut bins = list_bins(&dir.path().join("Cargo.toml")).expect("bins");
        bins.sort();
        assert_eq!(bins, vec!["alpha".to_string(), "beta".to_string()]);
    }
}
