use pru::{Pru, Procfile};
use std::io::{Read, Write};
use execute::shell;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::fs;

#[derive(Debug)]
struct Event {
    command: String,
    level: String,
    message: String,
}

pub fn run(args: Pru, mut writer: impl Write) -> Result<(), Box<dyn std::error::Error>> {
    let procfile_path = args.procfile;
    let procfile = match fs::read_to_string(&procfile_path) {
        Ok(contents) => Procfile::from(contents.as_str()),
        Err(e) => return Err(Box::new(e)),
    };
    let (tx, rx) = mpsc::channel::<Event>();

    // for each command in the procfile
    // * start the command
    // * have the command send all its output (and error) back to us
    for command in &procfile.commands {
        let (out, err) = producer(&command.command.clone());
        consumer(command.key.clone(), "info".to_string(), out, tx.clone());
        consumer(command.key.clone(), "error".to_string(), err, tx.clone());
    }

    // * loop over all received input and display it
    loop {
        // if let Ok(event) = rx.try_recv() {
        if let Ok(event) = rx.recv() {
            writeln!(writer, "{:?}", event)?;
        }
    }

    Ok(())
}

fn producer(command: &str) -> (impl Read, impl Read) {
    let sh = shell(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    (sh.stdout.unwrap(), sh.stderr.unwrap())
}

fn consumer(command: String, level: String, reader: impl std::io::Read + Send + 'static, tx: mpsc::Sender<Event>) {
    thread::spawn(move|| {
        listener(&command, level, reader, tx);
    });
}

fn listener(command: &String, level: String, reader: impl std::io::Read, tx: mpsc::Sender<Event>) {
    for line in BufReader::new(reader).lines() {
        tx.send(Event {
            command: command.clone(),
            level: level.clone(),
            message: line.unwrap(),
        })
        .unwrap();
    }
}
