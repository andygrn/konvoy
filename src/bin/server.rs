use std::io::{Read, Write, BufRead};
use std::io::Error;
use std::io::ErrorKind;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};

fn expect_response(stream: &mut Read, expected_response: &str) -> Result<(), ()> {
    let mut stream = stream.take(expected_response.len() as u64);
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).unwrap();
    if buffer != expected_response {
        return Err(())
    }
    Ok(())
}

fn send_archives(stream: TcpStream) -> Result<usize, Error> {
    let mut stream_r = BufReader::new(&stream);
    let mut stream_w = BufWriter::new(&stream);

    let response = expect_response(&mut stream_r, "SHARING?\n");
    if response.is_err() {
        return Err(Error::new(ErrorKind::ConnectionAborted, "Not sending updates!"))
    }

    stream_w.write("YES\n".as_bytes())?;
    stream_w.flush().unwrap();

    for line in stream_r.lines() {
        let line = line.unwrap();
        println!("{} - Sending update", &line);

        stream_w.write(format!("File content for {}", &line).as_bytes())?;
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
