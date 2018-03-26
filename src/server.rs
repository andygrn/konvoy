extern crate byteorder;
extern crate konvoy_archive;

use std::io::{BufRead, Write};
use std::io::Error;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::fs::File;
use byteorder::{BigEndian, WriteBytesExt};
use konvoy_archive::Archive;

fn send_archives(stream: TcpStream) -> Result<usize, Error> {
    let stream_r = BufReader::new(&stream);
    let mut stream_w = BufWriter::new(&stream);

    for line in stream_r.lines() {
        let line = line.unwrap();

        let archive = Archive::from_name(&line);

        if archive.is_err() {
            println!("{} Invalid request.", &line);
            stream_w.write_u64::<BigEndian>(0)?;
            stream_w.flush().unwrap();
            continue;
        }
        let archive = archive.unwrap();

        let mut archives = std::fs::read_dir("archives/server").unwrap();
        let archive_public_key_base64 = archive.get_public_key_base64();
        let existing_archive = archives.find(|existing_archive| {
            existing_archive
                .as_ref()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .starts_with(&archive_public_key_base64)
        });

        if existing_archive.is_none() {
            println!("{} Archive not here.", &archive_public_key_base64);
            stream_w.write_u64::<BigEndian>(0)?;
            stream_w.flush().unwrap();
            continue;
        }

        let archive_filename = &existing_archive
            .unwrap()
            .unwrap()
            .file_name()
            .into_string()
            .unwrap();
        let mut archive_file = File::open(format!("archives/server/{}", &archive_filename));
        if archive_file.is_err() {
            println!("{} Archive unavailable.", &line);
            stream_w.write_u64::<BigEndian>(0)?;
            stream_w.flush().unwrap();
            continue;
        }
        let existing_archive = Archive::from_stream(archive_filename, &mut archive_file.unwrap());
        if existing_archive.is_err() {
            println!("{} WTF.", &line);
            stream_w.write_u64::<BigEndian>(0)?;
            stream_w.flush().unwrap();
            continue;
        }
        let existing_archive = existing_archive.unwrap();

        if archive.datetime < existing_archive.datetime {
            println!("{} Sending update.", &line);
            stream_w.write_u64::<BigEndian>(existing_archive.get_data_size())?;
            stream_w.write(existing_archive.get_filename().as_bytes())?;
            stream_w.write(existing_archive.get_signature_base64().as_bytes())?;
            stream_w.write(&existing_archive.data)?;
        } else {
            println!("{} No update to send.", &line);
            stream_w.write_u64::<BigEndian>(0)?;
        }
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
            Err(e) => {
                println!("Client connection failed: {}", e);
            }
        }
    }
}
