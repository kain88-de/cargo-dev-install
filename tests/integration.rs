use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write file");
}

fn create_single_bin_crate(root: &Path) {
    write_file(
        &root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.rs\"\n",
    );
    write_file(
        &root.join("src/main.rs"),
        "fn main() { println!(\"ok\"); }\n",
    );
}

fn create_single_bin_crate_with_output(root: &Path) {
    write_file(
        &root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.rs\"\n",
    );
    write_file(
        &root.join("src/main.rs"),
        "fn main() {\n    println!(\"MARKER\");\n    for arg in std::env::args().skip(1) {\n        println!(\"arg:{arg}\");\n    }\n}\n",
    );
}

fn create_multi_bin_crate(root: &Path) {
    write_file(
        &root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"alpha\"\npath = \"src/main.rs\"\n\n[[bin]]\nname = \"beta\"\npath = \"src/bin/beta.rs\"\n",
    );
    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(&root.join("src/bin/beta.rs"), "fn main() {}\n");
}

fn run_plugin(
    cwd: &Path,
    home: &Path,
    path_var: &str,
    xdg_bin: Option<&Path>,
    args: &[&str],
) -> assert_cmd::assert::Assert {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cargo-dev-install");
    cmd.current_dir(cwd).env("HOME", home).env("PATH", path_var);

    if let Some(xdg) = xdg_bin {
        cmd.env("XDG_BIN_HOME", xdg);
    }

    cmd.args(args).assert()
}

#[test]
fn installs_wrapper_in_default_location() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    let path_var = "/usr/bin";

    run_plugin(repo.path(), home.path(), path_var, None, &[]).success();

    let wrapper_path = home.path().join(".local/bin/demo");
    assert!(wrapper_path.is_file());
    let contents = fs::read_to_string(&wrapper_path).expect("read wrapper");
    assert!(contents.contains(
        "exec cargo run --quiet --release --manifest-path \"$REPO/Cargo.toml\" -- \"$@\""
    ));
}

#[test]
fn installs_wrapper_in_xdg_bin_home() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    let xdg = tempfile::tempdir().expect("xdg");
    let path_var = "/usr/bin";

    run_plugin(repo.path(), home.path(), path_var, Some(xdg.path()), &[]).success();

    let wrapper_path = xdg.path().join("demo");
    assert!(wrapper_path.is_file());
}

#[test]
fn warns_when_install_dir_not_on_path() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    let path_var = "/usr/bin";

    run_plugin(repo.path(), home.path(), path_var, None, &[])
        .success()
        .stderr(predicate::str::contains(
            "Warning: install directory is not on PATH",
        ));
}

#[test]
fn does_not_warn_when_install_dir_on_path() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    let install_dir = home.path().join(".local/bin");
    let path_var = format!("{}:/usr/bin", install_dir.display());

    run_plugin(repo.path(), home.path(), &path_var, None, &[])
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn refuses_overwrite_without_force() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    let install_dir = home.path().join(".local/bin");
    fs::create_dir_all(&install_dir).expect("create install dir");
    let wrapper = install_dir.join("demo");
    fs::write(&wrapper, "echo old\n").expect("write wrapper");

    run_plugin(repo.path(), home.path(), "/usr/bin", None, &[])
        .failure()
        .stderr(predicate::str::contains("failed to write wrapper"));
}

#[test]
fn overwrites_with_force_flag() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    let install_dir = home.path().join(".local/bin");
    fs::create_dir_all(&install_dir).expect("create install dir");
    let wrapper = install_dir.join("demo");
    fs::write(&wrapper, "echo old\n").expect("write wrapper");

    run_plugin(repo.path(), home.path(), "/usr/bin", None, &["--force"]).success();
    let contents = fs::read_to_string(&wrapper).expect("read wrapper");
    assert!(contents.contains("REPO=\""));
}

#[test]
fn supports_multi_bin_with_bin_flag() {
    let repo = tempfile::tempdir().expect("repo");
    create_multi_bin_crate(repo.path());

    let home = tempfile::tempdir().expect("home");
    run_plugin(
        repo.path(),
        home.path(),
        "/usr/bin",
        None,
        &["--bin", "beta"],
    )
    .success();

    let wrapper = home.path().join(".local/bin/beta");
    assert!(wrapper.is_file());
}

#[test]
fn executes_installed_wrapper() {
    let repo = tempfile::tempdir().expect("repo");
    create_single_bin_crate_with_output(repo.path());

    let home = tempfile::tempdir().expect("home");
    let path_var = "/usr/bin";

    run_plugin(repo.path(), home.path(), path_var, None, &[]).success();

    let wrapper = home.path().join(".local/bin/demo");
    let output = Command::new(&wrapper)
        .current_dir(repo.path())
        .env("CARGO_TARGET_DIR", repo.path().join("target"))
        .output()
        .expect("run wrapper");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MARKER"));
    assert!(!stdout.contains("arg:demo"));

    let output = Command::new(&wrapper)
        .current_dir(repo.path())
        .env("CARGO_TARGET_DIR", repo.path().join("target"))
        .arg("hello")
        .output()
        .expect("run wrapper");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MARKER"));
    assert!(stdout.contains("arg:hello"));
}
