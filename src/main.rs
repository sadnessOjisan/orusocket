extern crate base64;
extern crate httparse;
extern crate hex;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    for stream in listener.incoming() {
        println!("======== ffff ffff =========");
        thread::spawn(move || {
            let s = stream.unwrap();
            handle_client(&s);
            println!("{:?}",s);
        loop {
            print_stream(&s);
        }
        });
    }
}
fn print_stream(mut stream: &TcpStream) {
    let mut buf = [0; 4096];
    println!("{:?}", stream);
    stream.read(&mut buf).unwrap();
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&buf).unwrap();
    println!("{:?}", res);
}

fn handle_client(mut stream: &TcpStream) {
    let key = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let clientCode = "function () {
  var ws = new WebSocket('ws://localhost:8080/', ['test', 'chat']);
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
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&buf).unwrap();
    if res.is_complete() {
        req.parse(&buf).unwrap();
        let is_upgrade = req
            .headers
            .iter()
            .find(|&&x| x.name == "Upgrade")
            .iter()
            .len();

        if (is_upgrade > 0) {
            let token_bytes = req
                .headers
                .iter()
                .find(|&&x| x.name == "Sec-WebSocket-Key")
                .unwrap()
                .value;
            let token_bytes_str = std::str::from_utf8(token_bytes).unwrap();
            let joined_token = &*(token_bytes_str.to_string() + key);
            let mut hasher = Sha1::new();
            hasher.input(joined_token.as_bytes());
            let sha1_string = hasher.result_str();
            // sha1_string: 558c6e2f93212d10f8b4ab1ac77031e2ba157471
            let bytes = hex::decode(sha1_string).unwrap();
            let sha1_base64 = base64::encode(bytes);

            let status = "HTTP/1.1 101 Switching Protocols\r\n".to_string();
            let header = status
                + "Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: "
                + &*sha1_base64
                + "
Sec-WebSocket-Protocol: chat\r\n\r\n";

            let res = header;
            println!("======== Response Header =========");
            println!("{}", res);
            println!("---------------------------");
            let data = res.as_bytes();
            stream.write(data);
        } else {
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
    }
}
