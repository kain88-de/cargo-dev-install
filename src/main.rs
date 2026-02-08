#[cfg(target_os = "linux")]
fn main() {
    if let Err(err) = cargo_dev_install::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("cargo-dev-install is only supported on Linux");
    std::process::exit(1);
}
