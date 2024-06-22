use argh::FromArgs;
use log::{error, info};
use phargs::*;

#[derive(Debug, PartialEq)]
struct Xargs(Vec<String>);

impl std::str::FromStr for Xargs {
    type Err = std::convert::Infallible;
    /// comma separated
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Xargs(
            s.split(',').map(|s| s.to_string()).collect::<Vec<_>>(),
        ))
    }
}

#[derive(FromArgs, PartialEq, Debug)]
/// Multiple command runner in one line
struct Args {
    #[argh(option, short = 'w')]
    /// comma separated arguments
    wlist: Xargs,

    #[argh(switch, short = 'n')]
    /// dry run
    dry_run: bool,

    /// actual running command
    #[argh(positional, greedy)]
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Args = argh::from_env();

    if opts.command.is_empty() {
        return Err("command is empty".into());
    }

    let mut command = opts.command;
    let args = command.split_off(1);

    let commands = PhCommandVec::new(&command[0], args, opts.wlist.0);

    for a in commands.iter() {
        if opts.dry_run {
            println!("{}", a.command_string());
            continue;
        }
        info!("running: {}", a.command_string());
        let status = a.command().status()?;
        if !status.success() {
            error!("failed to run: {}", a.command_string());
            std::process::exit(status.code().ok_or("exit code not found")?);
        }
    }

    Ok(())
}
