use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    bin: Option<String>,
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub bin: Option<String>,
    pub force: bool,
}

pub fn parse_args<I, T>(args: I) -> Result<CliArgs, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let mut args_iter = args.into_iter();
    let program = args_iter.next();
    let mut remaining: Vec<T> = args_iter.collect();

    if remaining
        .first()
        .map(|arg| arg.clone().into())
        .as_ref()
        .map(|arg| arg == "dev-install")
        .unwrap_or(false)
    {
        remaining.remove(0);
    }

    let mut argv = Vec::with_capacity(1 + remaining.len());
    if let Some(program) = program {
        argv.push(program);
    }
    argv.extend(remaining);

    let parsed = Args::try_parse_from(argv)?;
    Ok(CliArgs {
        bin: parsed.bin,
        force: parsed.force,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_args() {
        let args = parse_args(["cargo-dev-install"]).expect("parse args");
        assert_eq!(
            args,
            CliArgs {
                bin: None,
                force: false,
            }
        );
    }

    #[test]
    fn parses_bin_and_force() {
        let args =
            parse_args(["cargo-dev-install", "--bin", "demo", "--force"]).expect("parse args");
        assert_eq!(
            args,
            CliArgs {
                bin: Some("demo".to_string()),
                force: true,
            }
        );
    }

    #[test]
    fn parses_subcommand_invocation() {
        let args = parse_args(["cargo", "dev-install", "--force"]).expect("parse args");
        assert_eq!(
            args,
            CliArgs {
                bin: None,
                force: true,
            }
        );
    }

    #[test]
    fn errors_on_unknown_flag() {
        let err = parse_args(["cargo-dev-install", "--unknown"])
            .expect_err("expected error for unknown flag");
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }
}
