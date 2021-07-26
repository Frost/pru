use pru::pru_check;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "Run Procfile-based applications")]
enum Cmd {
    /// Validate your application's Procfile
    #[structopt(name = "check")]
    Check,
    /// Export the application to another process management format
    #[structopt(name = "export")]
    Export {
        /// What format to export
        format: String,
        /// Path to export the application to
        location: PathBuf,
    },
    /// Run a command using your application's environment
    #[structopt(name = "run")]
    Run {
        /// Command to run
        command: String,
        /// Args for the command
        args: Vec<String>,
    },
    /// Start the application (or a specific process)
    #[structopt(name = "start")]
    Start {
        /// Process to start
        process: Option<String>,
    },
    /// Display current version
    #[structopt(name = "version")]
    Version,
}

#[derive(StructOpt)]
struct Pru {
    #[structopt(subcommand)]
    cmd: Cmd,
    /// Path to your Procfile
    #[structopt(long, short = "-f", default_value = "Procfile")]
    procfile: PathBuf,
    /// Procfile directory
    #[structopt(long, short = "-d", default_value = ".")]
    root: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Pru::from_args();
    let out = std::io::stdout();
    let root = args.root;
    let procfile = args.procfile;

    match args.cmd {
        Cmd::Check => {
            pru_check(&root, procfile, out)?;
        }
        Cmd::Export { format, location } => {
            pru_export(root, procfile, format, location, out);
        }
        Cmd::Run { command, args } => {
            pru_run(root, procfile, command, args, out);
        }
        Cmd::Start { process } => {
            pru_start(root, procfile, process, out);
        }
        Cmd::Version => {
            pru_version(out);
        }
    }

    Ok(())
}

fn pru_export(
    _root: PathBuf,
    _procfile: PathBuf,
    _format: String,
    _location: PathBuf,
    mut _writer: impl std::io::Write,
) {
    not_yet_implemented();
}

fn pru_run(
    _root: PathBuf,
    _procfile: PathBuf,
    _command: String,
    _args: Vec<String>,
    mut _writer: impl std::io::Write,
) {
    not_yet_implemented();
}

fn pru_start(
    _root: PathBuf,
    _procfile: PathBuf,
    _process: Option<String>,
    mut _writer: impl std::io::Write,
) {
    not_yet_implemented();
}

fn pru_version(mut writer: impl std::io::Write) {
    writeln!(writer, "pru {}", env!("CARGO_PKG_VERSION")).unwrap();
}

fn not_yet_implemented() {
    eprintln!("Not yet implemented!");
}
