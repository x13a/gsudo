use std::env;
use std::error;
use std::ffi::OsStr;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::result;

use security_framework::authorization::{
    Authorization, AuthorizationItemSetBuilder, Flags as AuthorizationFlags,
};

const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE: i32 = 2;

mod flag {
    pub const HELP: &'static str = "h";
    pub const VERSION: &'static str = "V";
}

enum PrintDestination {
    Stdout,
    Stderr,
}

fn print_usage(to: PrintDestination) {
    let prog_name = PathBuf::from(env::args_os().next().unwrap())
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let usage = format!(
        "{N} [-{h}|{V}] <EXECUTABLE> [..<ARG>]",
        N = prog_name,
        h = flag::HELP,
        V = flag::VERSION,
    );
    match to {
        PrintDestination::Stdout => println!("{}", usage),
        PrintDestination::Stderr => eprintln!("{}", usage),
    }
}

#[derive(Default)]
struct Opts {
    executable: PathBuf,
    args: Vec<String>,
}

fn exit_usage(s: impl AsRef<str>) {
    eprintln!("{}", s.as_ref());
    exit(EXIT_USAGE);
}

fn get_opts() -> Result<Opts> {
    let mut argv = env::args().skip(1);
    if argv.len() == 0 {
        print_usage(PrintDestination::Stderr);
        exit(EXIT_USAGE);
    }
    let mut opts = Opts::default();
    while let Some(arg) = argv.next() {
        if !arg.starts_with('-') {
            opts.executable = arg.into();
            break;
        }
        match arg[1..].as_ref() {
            flag::HELP => {
                print_usage(PrintDestination::Stdout);
                exit(EXIT_SUCCESS);
            }
            flag::VERSION => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                exit(EXIT_SUCCESS);
            }
            _ => {}
        }
    }
    if !opts.executable.starts_with("/") {
        exit_usage("absolute path required");
    }
    opts.args = argv.collect();
    Ok(opts)
}

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn exec(executable: impl AsRef<Path>, args: &[impl AsRef<OsStr>]) -> Result<()> {
    let rights = AuthorizationItemSetBuilder::new()
        .add_right("system.privilege.admin")?
        .build();
    let auth = Authorization::new(
        Some(rights),
        None,
        AuthorizationFlags::DEFAULTS
            | AuthorizationFlags::INTERACTION_ALLOWED
            | AuthorizationFlags::PREAUTHORIZE
            | AuthorizationFlags::EXTEND_RIGHTS,
    )?;
    let file =
        auth.execute_with_privileges(executable.as_ref(), args, AuthorizationFlags::DEFAULTS)?;
    for line in io::BufReader::new(file).lines() {
        if let Ok(s) = line {
            println!("{}", s);
        }
    }
    auth.destroy_rights();
    Ok(())
}

fn main() -> Result<()> {
    let opts = get_opts()?;
    exec(opts.executable, &opts.args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_ls() {
        exec("/bin/ls", &["-la"]).expect("ok")
    }
}
