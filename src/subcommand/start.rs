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

pub fn run(args: Pru, mut out: impl Write, mut err: impl Write) -> Result<(), Box<dyn std::error::Error>> {
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
        let (cmdout, cmderr) = producer(&command.command.clone());
        consumer(command.key.clone(), "stdout".to_string(), cmdout, tx.clone());
        consumer(command.key.clone(), "stderr".to_string(), cmderr, tx.clone());
    }

    // * loop over all received input and display it
    loop {
        // if let Ok(event) = rx.try_recv() {
        if let Ok(event) = rx.recv() {
            let message = format!("{} [{}] {}", event.command, event.level, event.message);
            match event.level.as_str() {
                "stdout" => writeln!(out, "{}", &message)?,
                "stderr" => writeln!(err, "{}", &message)?,
                &_ => continue,
            };
        }
    }

    Ok(())
}

fn producer(command: &str) -> (impl Read, impl Read) {
    let sh = shell(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        // Do we need to catch this in some other way?
        // How to handle when sub processes die?
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
