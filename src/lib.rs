pub mod cli;
pub mod install;
pub mod project;
pub mod tui_select;

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub crate_root: PathBuf,
    pub manifest_path: PathBuf,
    pub bin_name: String,
    pub install_dir: PathBuf,
    pub wrapper_path: PathBuf,
    pub wrapper_contents: String,
    pub warn_path_missing: bool,
}

#[derive(Debug, Clone)]
pub struct EnvSnapshot {
    pub home: Option<PathBuf>,
    pub xdg_bin_home: Option<PathBuf>,
    pub path: Option<String>,
}

impl EnvSnapshot {
    pub fn capture() -> Self {
        Self {
            home: std::env::var_os("HOME").map(PathBuf::from),
            xdg_bin_home: std::env::var_os("XDG_BIN_HOME").map(PathBuf::from),
            path: std::env::var("PATH").ok(),
        }
    }
}

pub fn run() -> Result<(), String> {
    let args = cli::parse_args(std::env::args()).map_err(|err| err.to_string())?;
    let env = EnvSnapshot::capture();
    let cwd = std::env::current_dir().map_err(|err| format!("failed to read cwd: {err}"))?;
    let plan = make_plan(&args, &env, &cwd)?;
    apply_plan(&plan, args.force)
}

pub fn make_plan(
    args: &cli::CliArgs,
    env: &EnvSnapshot,
    cwd: &Path,
) -> Result<InstallPlan, String> {
    let crate_root = project::find_crate_root(cwd)?;
    let manifest_path = crate_root.join("Cargo.toml");

    let bin_names = project::list_bins(&manifest_path)?;
    let bin_name = select_bin(args, &bin_names)?;

    let install_dir = install::install_dir(install::EnvSnapshot {
        home: env.home.as_deref(),
        xdg_bin_home: env.xdg_bin_home.as_deref(),
        path: env.path.as_deref(),
    })
    .ok_or_else(|| "HOME is not set; cannot determine install directory".to_string())?;

    let wrapper_path = install_dir.join(&bin_name);
    let wrapper_contents = install::render_wrapper(&crate_root);
    let warn_path_missing = !install::is_on_path(&install_dir, env.path.as_deref());

    Ok(InstallPlan {
        crate_root,
        manifest_path,
        bin_name,
        install_dir,
        wrapper_path,
        wrapper_contents,
        warn_path_missing,
    })
}

pub fn apply_plan(plan: &InstallPlan, force: bool) -> Result<(), String> {
    install::write_wrapper(&plan.wrapper_path, &plan.wrapper_contents, force)
        .map_err(|err| format!("failed to write wrapper: {err}"))?;

    if plan.warn_path_missing {
        eprintln!("Warning: install directory is not on PATH");
        eprintln!("Add it to your shell profile, e.g.:");
        eprintln!("export PATH=\"{}:$PATH\"", plan.install_dir.display());
    }

    Ok(())
}

fn select_bin(args: &cli::CliArgs, bin_names: &[String]) -> Result<String, String> {
    if bin_names.is_empty() {
        return Err("no binary targets found in Cargo.toml".to_string());
    }

    if bin_names.len() == 1 {
        return Ok(bin_names[0].clone());
    }

    if let Some(bin) = &args.bin {
        if bin_names.iter().any(|name| name == bin) {
            return Ok(bin.clone());
        }
        return Err(format!("binary '{bin}' not found in crate"));
    }

    if std::io::stdin().is_terminal() {
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();
        return tui_select::select_bin(bin_names, &mut stdin, &mut stdout)
            .map_err(|err| format!("failed to select binary: {err}"));
    }

    Err("multiple binaries found; pass --bin <name>".to_string())
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

    fn default_env(home: &Path, path_var: &str) -> EnvSnapshot {
        EnvSnapshot {
            home: Some(home.to_path_buf()),
            xdg_bin_home: None,
            path: Some(path_var.to_string()),
        }
    }

    #[test]
    fn make_plan_single_bin() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.rs\"\n",
        );
        write_file(&dir.path().join("src/main.rs"), "fn main() {}\n");

        let args = cli::CliArgs {
            bin: None,
            force: false,
        };
        let env = default_env(dir.path(), "/usr/bin");

        let plan = make_plan(&args, &env, dir.path()).expect("plan");
        assert_eq!(plan.bin_name, "demo");
        assert_eq!(plan.manifest_path, dir.path().join("Cargo.toml"));
        assert_eq!(plan.wrapper_path, dir.path().join(".local/bin/demo"));
        assert!(plan.wrapper_contents.contains("REPO=\""));
    }

    #[test]
    fn make_plan_respects_bin_flag() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"alpha\"\npath = \"src/main.rs\"\n\n[[bin]]\nname = \"beta\"\npath = \"src/bin/beta.rs\"\n",
        );
        write_file(&dir.path().join("src/main.rs"), "fn main() {}\n");
        write_file(&dir.path().join("src/bin/beta.rs"), "fn main() {}\n");

        let args = cli::CliArgs {
            bin: Some("beta".to_string()),
            force: false,
        };
        let env = default_env(dir.path(), "/usr/bin");

        let plan = make_plan(&args, &env, dir.path()).expect("plan");
        assert_eq!(plan.bin_name, "beta");
    }

    #[test]
    fn make_plan_warns_when_path_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(
            &dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.rs\"\n",
        );
        write_file(&dir.path().join("src/main.rs"), "fn main() {}\n");

        let args = cli::CliArgs {
            bin: None,
            force: false,
        };
        let env = default_env(dir.path(), "/usr/bin");

        let plan = make_plan(&args, &env, dir.path()).expect("plan");
        assert!(plan.warn_path_missing);
    }
}
