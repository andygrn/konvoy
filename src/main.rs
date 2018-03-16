use std::io::{Read, Write};
use std::io::Error;
use std::io::ErrorKind;
use std::io::BufWriter;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::fs::File;

fn expect_response(stream: &TcpStream, expected_response: &str) -> Result<(), ()> {
    let mut stream = stream.take(expected_response.len() as u64);
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).unwrap();
    if buffer != expected_response {
        return Err(())
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<usize, Error> {
    // let mut buffer = [0; 10];
    // let read = stream.read(&mut buffer[..]).unwrap_or(0);
    // println!("{} bytes were read", read);
    stream.write("Will u send updates?\n".as_bytes())?;

    let response = expect_response(&stream, "YEE\n");
    if response.is_err() {
        return Err(Error::new(ErrorKind::ConnectionAborted, "Not sending updates!"))
    }

    stream.write("Send me updates for phil, bill n jill plz.\n".as_bytes())?;

    let response = expect_response(&stream, "HERE\n");
    if response.is_err() {
        return Err(Error::new(ErrorKind::ConnectionAborted, "Invalid updates!"))
    }

    stream.write("Thanks bye!\n".as_bytes())?;

    // let mut file = BufWriter::new(File::create("foo.txt").unwrap());
    // for byte in stream.bytes() {
    //     file.write(&[byte.unwrap()]).unwrap();
    // }
    // println!("Wrote to file foo.txt");

    Ok(0)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9123").unwrap();
    println!("Server listening!");
    // match listener.accept() {
    //     Ok((_socket, addr)) => println!("new client: {:?}", addr),
    //     Err(e) => println!("couldn't get client: {:?}", e),
    // }
    // for stream in listener.incoming() {
    //     thread::spawn(|| {
    //         let mut stream = stream.unwrap();
    //         println!("Got connection!");
    //         stream.write(b"Hello World\r\n").unwrap();
    //     });
    // }
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected.");
                match handle_client(stream) {
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
