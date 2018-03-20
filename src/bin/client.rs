extern crate chrono;

use std::io::{BufRead, Read, Write};
use std::io::Error;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::fs::File;

// use chrono::{DateTime, Utc};

fn request_archives(stream: TcpStream) -> Result<usize, Error> {
    let mut stream_r = BufReader::new(&stream);
    let mut stream_w = BufWriter::new(&stream);

    // let now: DateTime<Utc> = Utc::now();

    let following = BufReader::new(File::open("following.txt").unwrap());
    for id in following.lines() {
        let id = id.unwrap();

        let mut archives = std::fs::read_dir("archives_client").unwrap();
        let existing_archive = archives.find(|archive| {
            archive
                .as_ref()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .starts_with(&id)
        });

        if existing_archive.is_none() {
            stream_w.write(format!("{}@0000000000\n", &id).as_bytes())?;
        } else {
            // Ask for updates to the followed archive.
            let file_name = &existing_archive
                .unwrap()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap();
            stream_w.write(format!("{}\n", file_name).as_bytes())?;
        }
        stream_w.flush().unwrap();

        {
            // Skip file creation if server has no update.
            let mut peekbuf = [0; 1];
            stream.peek(&mut peekbuf).unwrap();
            if peekbuf[0] == 0x0 {
                // No archive sent.
                stream_r.read(&mut peekbuf).unwrap();
                println!("{} No update available.", &id);
                continue;
            }
        }

        {
            // Download update into the archive file.
            let mut file = BufWriter::new(
                File::create(format!("archives/client/{}@{}", &id, "1521242772")).unwrap(),
            );
            for byte in stream_r.by_ref().bytes() {
                let byte = byte.unwrap();
                if byte == 0x0 {
                    // End of file reached.
                    break;
                }
                file.write(&[byte]).unwrap();
                file.flush().unwrap();
            }
            println!("{} Updated.", &id);
        }
    }

    Ok(0)
}

fn main() {
    let stream = TcpStream::connect("127.0.0.1:9123");
    match stream {
        Ok(stream) => {
            println!("Connected to server.");
            match request_archives(stream) {
                Ok(_exit_code) => {
                    println!("Received updates.");
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
            println!("Disconnected from server.");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
