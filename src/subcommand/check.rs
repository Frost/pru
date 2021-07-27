use pru::{Pru, Procfile};
use std::io::{Error, ErrorKind};
use std::io::Write;
use std::fs;

pub fn run(
    args: Pru,
    mut writer: impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let procfile_path = args.procfile;
    let procfile = match fs::read_to_string(&procfile_path) {
        Ok(contents) => Procfile::from(contents.as_str()),
        Err(_e) => {
            let error_message = format!(
                "ERROR: Procfile does not exist: {}",
                procfile_path.display()
            );
            return Err(error(ErrorKind::NotFound, &error_message));
        }
    };

    if procfile.commands.len() < 1 {
        return Err(error(ErrorKind::Other, "ERROR: no processes defined"));
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

fn error(kind: ErrorKind, message: &str) -> std::boxed::Box<std::io::Error> {
    Box::new(Error::new(kind, message))
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
