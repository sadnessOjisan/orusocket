extern crate httparse;

use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use std::thread;
use std::fs::File;
use std::path::Path;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    for stream in listener.incoming() {
        thread::spawn(|| {
            handle_client(stream.unwrap())
        });
    }
}


fn handle_client(mut stream: TcpStream) {
    let mut buf = [0 ;4096];

    stream.read(&mut buf).unwrap();
    let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut parsed_headers);
    req.parse(&buf).unwrap();
    match req.path {
        Some(ref path) => {
            let mut body = "hello world";
            let status = "HTTP/1.1 200 OK\r\n".to_string();
            let header = status + "Content-Type: text/html; charset=UTF-8\r\n\r\n";
            let res = header + &body + "\r\n";
            let data = res.as_bytes();
            stream.write(data);
        },
        None => {
        }
    }
}

