extern crate regex;

use std::io::{Write, BufRead};
use std::io::Error;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};

use regex::Regex;

#[derive(Debug)]
struct ArchiveName {
    id: String,
    date: String,
}

fn parse_archive_name(name: &str) -> Result<ArchiveName, ()> {
    let re = Regex::new(r"(?x)
        ^(
            [0-9a-f]{8}- #
            [0-9a-f]{4}- #
            [0-9a-f]{4}- # GUID
            [0-9a-f]{4}- #
            [0-9a-f]{12} #
        )@(
            \d{10}       # Unix timestamp
        )$
    ").unwrap();

    let captures = re.captures(name);
    if captures.is_none() {
        return Err(())
    }

    let captures = captures.unwrap();
    let archive_id = captures.get(1);
    let archive_date = captures.get(2);
    if archive_id.is_none() || archive_date.is_none() {
        return Err(())
    }

    Ok(ArchiveName {
        id: archive_id.unwrap().as_str().to_string(),
        date: archive_date.unwrap().as_str().to_string(),
    })
}

fn send_archives(stream: TcpStream) -> Result<usize, Error> {
    let stream_r = BufReader::new(&stream);
    let mut stream_w = BufWriter::new(&stream);

    for line in stream_r.lines() {
        let line = line.unwrap();

        let name = parse_archive_name(&line);

        if name.is_err() {
            println!("{} Invalid request.", &line);
            stream_w.write(&[0x0_u8])?; // Send EOF marker.
            stream_w.flush().unwrap();
            continue;
        }
        let name = name.unwrap();

        let mut archives = std::fs::read_dir("archives_server").unwrap();
        let existing_archive = archives.find(|archive| {
            archive.as_ref().unwrap().file_name().into_string().unwrap().starts_with(&name.id)
        });

        if existing_archive.is_none() {
            println!("{} Archive not here.", &line);
            stream_w.write(&[0x0_u8])?; // Send EOF marker.
            stream_w.flush().unwrap();
            continue;
        }

        let existing_name = parse_archive_name(&existing_archive.unwrap().unwrap().file_name().into_string().unwrap());
        if existing_name.is_err() {
            println!("{} WTF.", &line);
            stream_w.write(&[0x0_u8])?; // Send EOF marker.
            stream_w.flush().unwrap();
            continue;
        }
        let existing_name = existing_name.unwrap();

        if name.date < existing_name.date {
            stream_w.write(format!("File content for {}", &line).as_bytes())?;
            println!("{} Sent update.", &line);
        } else {
            println!("{} No update to send.", &line);
        }
        stream_w.write(&[0x0_u8])?; // Send EOF marker.
        stream_w.flush().unwrap();
    }

    Ok(0)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9123").unwrap();
    println!("Server listening!");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected.");
                match send_archives(stream) {
                    Ok(_exit_code) => {
                        println!("Client received updates.");
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
                println!("Client disconnected.");
            }
            Err(_e) => {
                println!("Client connection failed.");
            }
        }
    }
}
