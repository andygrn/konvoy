extern crate byteorder;
extern crate konvoy_archive;

use std::io::{BufRead, Read, Write};
use std::io::Error;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::fs;
use std::fs::File;
use std::path::Path;
use byteorder::{BigEndian, ReadBytesExt};
use konvoy_archive::Archive;

fn request_archives(stream: TcpStream) -> Result<usize, Error> {
    const FILENAME_VERSION: u8 = 1;
    const HEADER_SIZE: usize = 142;
    let mut stream_r = BufReader::new(&stream);
    let mut stream_w = BufWriter::new(&stream);

    let following = BufReader::new(File::open("following.txt").unwrap());
    for id in following.lines() {
        let id = id.unwrap();

        if id == "" || id.starts_with('#') {
            continue;
        }

        let mut archives = std::fs::read_dir("archives/client").unwrap();
        let existing_archive = archives.find(|archive| {
            archive
                .as_ref()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .starts_with(&id)
        });

        let mut file_name;
        let mut file_exists = false;
        if existing_archive.is_none() {
            file_name = format!("{}{:03}000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", &id, &FILENAME_VERSION);
        } else {
            file_exists = true;
            // Ask for updates to the followed archive.
            file_name = existing_archive
                .unwrap()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap();
        }
        stream_w.write(format!("{}\n", &file_name).as_bytes())?;
        stream_w.flush().unwrap();

        {
            let archive_data_size = stream_r.read_u64::<BigEndian>().unwrap();
            if archive_data_size == 0 {
                // No archive sent.
                println!("{} No update available.", &id);
                continue;
            }

            println!("Receiving archive of size {} bytes.", &archive_data_size);

            // Download update into the archive file.
            let mut filename_buf = [0; HEADER_SIZE];
            stream_r.read(&mut filename_buf).unwrap();
            let filename = String::from_utf8(filename_buf.to_vec()).unwrap();
            let archive = Archive::from_stream(
                &filename,
                &mut stream_r.by_ref().take(archive_data_size as u64),
            );
            if archive.is_err() {
                println!("{} Corrupted archive received.", &id);
                continue;
            }
            let archive = archive.unwrap();
            if !archive.verify() {
                println!("{} Archive has been tampered with.", &id);
                continue;
            }
            let disk_write = archive.write_to_disk(Path::new("archives/client"));
            if disk_write.is_err() {
                println!("{} Failed to write archive to disk.", &id);
                continue;
            }
            if file_exists {
                fs::remove_file(format!("archives/client/{}", file_name))?;
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
