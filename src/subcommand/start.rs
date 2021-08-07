use pru::{Pru, Procfile};
use std::io::{Read, Write};
use execute::shell;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::fs;
use crossterm::style::{Color, Print, style};

use crossterm::execute;


#[derive(Debug)]
struct Event {
    command: String,
    message: String,
    color: Color,
}

impl Event {
    fn print(&self, mut stream: impl Write) -> Result<(), crossterm::ErrorKind> {
        execute!(
            stream,
            Print(style(&self.command).with(self.color)),
            Print(format!(" {}\n", &self.message)),
        )
    }
}

pub fn run(args: Pru, mut out: impl Write) -> Result<(), Box<dyn std::error::Error>> {
    let procfile_path = args.procfile;
    let procfile = match fs::read_to_string(&procfile_path) {
        Ok(contents) => Procfile::from(contents.as_str()),
        Err(e) => return Err(Box::new(e)),
    };
    let (tx, rx) = mpsc::channel::<Event>();

    let colors = &[
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Grey,
        Color::Yellow,
        Color::White,
        Color::DarkGrey,
        Color::DarkRed,
        Color::DarkGreen,
        Color::DarkBlue,
        Color::DarkMagenta,
        Color::DarkCyan,
        Color::DarkYellow,
    ];

    // for each command in the procfile
    // * start the command
    // * have the command send all its output (and error) back to us
    for (index, command) in procfile.commands.iter().enumerate() {
        let color = colors[index % colors.len()];
        let (cmdout, cmderr) = producer(&command.command.clone());
        consumer(command.key.clone(), cmdout, tx.clone(), color);
        consumer(command.key.clone(), cmderr, tx.clone(), color);
    }

    // * loop over all received input and display it
    loop {
        if let Ok(event) = rx.recv() {
            event.print(&mut out)?;
        }
    }
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

fn consumer(command: String, reader: impl std::io::Read + Send + 'static, tx: mpsc::Sender<Event>, color: Color) {
    thread::spawn(move|| {
        listener(&command, reader, tx, color);
    });
}

fn listener(command: &String, reader: impl std::io::Read, tx: mpsc::Sender<Event>, color: Color) {
    for line in BufReader::new(reader).lines() {
        tx.send(Event {
            command: command.clone(),
            message: line.unwrap(),
            color: color,
        })
        .unwrap();
    }
}
