use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "Run Procfile-based applications")]
pub enum Cmd {
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
pub struct Pru {
    #[structopt(subcommand)]
    pub cmd: Cmd,
    /// Path to your Procfile
    #[structopt(long, short = "-f", default_value = "Procfile")]
    pub procfile: PathBuf,
    /// Procfile directory
    #[structopt(long, short = "-d", default_value = ".")]
    pub root: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct SystemCommand {
    key: String,
    command: String,
}

impl From<&str> for SystemCommand {
    fn from(c: &str) -> SystemCommand {
        let parts = c.split(":").collect::<Vec<&str>>();
        SystemCommand {
            key: parts[0].to_string(),
            command: parts[1].trim().to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Procfile {
    commands: Vec<SystemCommand>,
}

impl From<&str> for Procfile {
    fn from(contents: &str) -> Procfile {
        let commands = contents
            .lines()
            .filter(|line| !line.starts_with("#"))
            .filter(|line| !line.trim().is_empty())
            .map(|line| SystemCommand::from(line))
            .collect();

        Procfile { commands: commands }
    }
}

impl Procfile {
    pub fn valid(&self) -> bool {
        self.commands.len() > 0
    }
}

pub fn pru_check(
    args: Pru,
    mut writer: impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    // pub fn pru_check(_procfile_dir: &PathBuf, procfile_path: PathBuf, mut writer: impl std::io::Write) -> Result<(), Box<dyn std::error::Error>> {
    let procfile_path = args.procfile;
    let procfile = match fs::read_to_string(&procfile_path) {
        Ok(contents) => Procfile::from(contents.as_str()),
        Err(_e) => {
            let error_message = format!(
                "ERROR: Procfile does not exist: {}",
                procfile_path.display()
            );
            return Err(Box::new(Error::new(ErrorKind::NotFound, error_message)));
        }
    };

    if procfile.commands.len() < 1 {
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            "ERROR: no processes defined",
        )));
    }

    let mut valid_commands = vec![];
    for command in &procfile.commands {
        valid_commands.push(String::from(&command.key));
    }
    writeln!(
        writer,
        "valid procfile detected ({})",
        &valid_commands.join(", ")
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::error::Error;
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    #[test]
    fn a_procfile_can_be_parsed() -> Result<(), Box<dyn std::error::Error>> {
        let file = NamedTempFile::new()?;
        let file_path = file.path().to_str().unwrap();
        writeln!(&file, "foo: ./foo")?;

        let mut cmd = Command::cargo_bin("pru")?;

        cmd.args(&["-f", file_path]).arg("check");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("valid procfile detected (foo)"));

        Ok(())
    }

    #[test]
    fn empty_procfile_displays_an_error() -> Result<(), Box<dyn Error>> {
        let file = NamedTempFile::new()?;
        let file_path = file.path().to_str().unwrap();

        let mut cmd = Command::cargo_bin("pru")?;

        cmd.args(&["-f", file_path]).arg("check");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("ERROR: no processes defined"));

        Ok(())
    }

    #[test]
    fn non_existing_proc_files_generate_error() -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::cargo_bin("pru")?;

        cmd.args(&["-f", "/some/non-existing/test/file"])
            .arg("check");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Procfile does not exist"));

        Ok(())
    }
}

#[cfg(test)]
mod procfile_struct_tests {
    use super::*;

    #[test]
    fn an_empty_procfile_is_not_valid() {
        let empty_procfile = Procfile { commands: vec![] };
        assert_eq!(empty_procfile.valid(), false);
    }

    #[test]
    fn a_procfile_with_commands_is_valid() {
        let procfile = Procfile {
            commands: vec![SystemCommand {
                key: "foo".to_string(),
                command: "foo".to_string(),
            }],
        };

        assert_eq!(procfile.valid(), true);
    }

    #[test]
    fn parse_a_system_command_from_string() {
        let command = SystemCommand::from("foo: ./foo");

        assert_eq!(command.key, "foo");
        assert_eq!(command.command, "./foo");
    }

    #[test]
    fn parse_a_procfile_from_string() {
        let procfile = Procfile::from("foo: ./foo\nbar:  ./bar");

        assert_eq!(procfile.commands.len(), 2);
        assert_eq!(procfile.commands[0], SystemCommand::from("foo: ./foo"));
        assert_eq!(procfile.commands[1], SystemCommand::from("bar: ./bar"));
    }

    #[test]
    fn parsing_procfiles_ignores_blank_lines() {
        let procfile = Procfile::from("\nfoo: ./foo");

        assert_eq!(procfile.commands[0], SystemCommand::from("foo: ./foo"));
    }

    #[test]
    fn parsing_procfiles_ignores_hash_comments() {
        let procfile = Procfile::from("#a comment\nfoo: ./foo");

        assert_eq!(procfile.commands[0], SystemCommand::from("foo: ./foo"));
    }
}
