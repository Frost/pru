use pru::{Cmd, Pru};
use structopt::StructOpt;
mod subcommand;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Pru::from_args();
    let out = std::io::stdout();

    match args.cmd {
        Cmd::Check { .. } => subcommand::check::run(args, out)?,
        Cmd::Export { .. } => not_yet_implemented(args, out),
        Cmd::Run { .. } => not_yet_implemented(args, out),
        Cmd::Start { .. } => not_yet_implemented(args, out),
        Cmd::Version => pru_version(args, out),
    }

    Ok(())
}

fn pru_version(_args: Pru, mut writer: impl std::io::Write) {
    writeln!(writer, "pru {}", env!("CARGO_PKG_VERSION")).unwrap();
}

fn not_yet_implemented(_args: Pru, mut _writer: impl std::io::Write) {
    eprintln!("Not yet implemented!");
}
