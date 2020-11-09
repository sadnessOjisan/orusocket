extern crate httparse;

use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    for stream in listener.incoming() {
        thread::spawn(|| handle_client(stream.unwrap()));
    }
}

fn handle_client(mut stream: TcpStream) {
    let clientCode = "function () {
  var ws = new WebSocket('ws://localhost:8080/', ['test', 'chat']);
  // var ws = new WebSocket('ws://localhost:8080/', 'test');
  ws.onopen = function() {
    console.log(ws);
    ws.send('test');
    ws.onmessage = function(message) {
      console.log(message.data);
    };
  }
}";
    let mut buf = [0; 4096];

    stream.read(&mut buf).unwrap();
    let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut parsed_headers);
    req.parse(&buf).unwrap();
    println!("{:?}", req.headers);
    println!("{:?}", req.method);
    let is_upgrade = req
        .headers
        .iter()
        .find(|&&x| x.name == "Upgrade")
        .iter()
        .len();
    println!("{}", is_upgrade);
    match req.path {
        Some(ref path) => {
            let mut body =
                "<html><head><title>rust web socket</title><script type='text/javascript'>("
                    .to_string()
                    + clientCode
                    + ")()</script></head><body>hello world!!!!!</body></html>";
            let status = "HTTP/1.1 200 OK\r\n".to_string();
            let header = status + "Content-Type: text/html; charset=UTF-8\r\n\r\n";
            let res = header + &body + "\r\n";
            let data = res.as_bytes();
            stream.write(data);
        }
        None => {}
    }
}
