use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Cmd {
    /// Validate your application's Procfile
    #[clap(name = "check")]
    Check,
    /// Export the application to another process management format
    #[clap(name = "export")]
    Export {
        /// What format to export
        format: String,
        /// Path to export the application to
        location: PathBuf,
    },
    /// Run a command using your application's environment
    #[clap(name = "run")]
    Run {
        /// Command to run
        command: String,
        /// Args for the command
        args: Vec<String>,
    },
    /// Start the application (or a specific process)
    #[clap(name = "start")]
    Start {
        /// Process to start
        process: Option<String>,
    },
    /// Display current version
    #[clap(name = "version")]
    Version,
}

#[derive(Parser,Debug)]
#[clap(version, about)]
pub struct Pru {
    #[clap(subcommand)]
    pub cmd: Cmd,
    /// Path to your Procfile
    #[clap(long, short = 'f', default_value = "Procfile")]
    pub procfile: PathBuf,
    /// Procfile directory
    #[clap(long, short = 'd', default_value = ".")]
    pub root: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct SystemCommand {
    pub key: String,
    pub command: String,
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
    pub commands: Vec<SystemCommand>,
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
