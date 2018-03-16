use std::io::{Read, Write, BufRead};
use std::io::Error;
use std::io::ErrorKind;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::fs::File;

fn expect_response(stream: &mut Read, expected_response: &str) -> Result<(), ()> {
    let mut stream = stream.take(expected_response.len() as u64);
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).unwrap();
    if buffer != expected_response {
        return Err(())
    }
    Ok(())
}

fn request_archives(stream: TcpStream) -> Result<usize, Error> {
    let mut stream_r = BufReader::new(&stream);
    let mut stream_w = BufWriter::new(&stream);

    stream_w.write("SHARING?\n".as_bytes())?;
    stream_w.flush().unwrap();

    let response = expect_response(&mut stream_r, "YES\n");
    if response.is_err() {
        return Err(Error::new(ErrorKind::ConnectionAborted, "Not sending updates!"))
    }

    let following = BufReader::new(File::open("following.txt").unwrap());
    for line in following.lines() {
        let line = line.unwrap();

        // Ask for updates to the followed archive.
        stream_w.write(format!("{}\n", &line).as_bytes())?;
        stream_w.flush().unwrap();
        println!("{} - Requested update", &line);

        {
            // Skip file creation if server has no update.
            let mut peekbuf = [0; 1];
            stream.peek(&mut peekbuf).unwrap();
            if peekbuf[0] == 0x0 {
                // No archive sent.
                stream_r.read(&mut peekbuf).unwrap();
                println!("{} - No update available", &line);
                continue;
            }
        }

        {
            // Download update into the archive file.
            let mut file = BufWriter::new(
                File::create(format!("archives/{}.txt", &line)).unwrap()
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
            println!("{} - Received update", &line);
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
                    println!("{}", e);
                }
            }
            println!("Disconnected from server.");
        }
        Err(_e) => {
            println!("Server connection failed.");
        }
    }
}
