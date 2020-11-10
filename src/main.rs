extern crate base64;
extern crate httparse;
extern crate rustc_serialize;

use base64::{encode};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::symmetriccipher::SynchronousStreamCipher;
use std::iter::repeat;
use rustc_serialize::base64::{STANDARD, ToBase64};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    for stream in listener.incoming() {
        thread::spawn(|| handle_client(stream.unwrap()));
    }
}

fn handle_client(mut stream: TcpStream) {
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
            let mut hasher = Sha1::new();
            let token_bytes = req
                .headers
                .iter()
                .find(|&&x| x.name == "Sec-WebSocket-Key")
                .unwrap()
                .value;
            let token_bytes_str = std::str::from_utf8(token_bytes).unwrap();
            println!("token_bytes_str: {:?}", token_bytes_str);
            let joined_token = &*(token_bytes_str.to_string() + key);
            let mut joined_token_byte = joined_token.as_bytes().to_vec();
            hasher.result(&mut joined_token_byte[..]);
            let based = encode(joined_token_byte.to_base64(STANDARD));

            // let mut bytes = repeat(0u8).take(hasher.output_bytes()).collect();
            // hasher.result(&mut bytes[..]);
            // println!("{}", bytes.to_base64(STANDARD));





            let status = "HTTP/1.1 101 Switching Protocols\r\n".to_string();
            let header =
                status+
                "Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: "+ &*based+
                "
Sec-WebSocket-Protocol: chat";
            
            let res = header;
            println!("res: {}", res);
            let data = res.as_bytes();
            stream.write(data);
            return;
        }
    }
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
            // println!("{}", res);
            let data = res.as_bytes();
            stream.write(data);
        }
        None => {}
    }
}