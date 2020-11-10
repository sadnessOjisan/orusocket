extern crate base64;
extern crate httparse;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use crypto::digest::Digest;
use crypto::sha1::Sha1;

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
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&buf).unwrap();
    if res.is_complete() {
        req.parse(&buf).unwrap();
        println!("{:?}", req.headers);
        println!("{:?}", req.method);
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
            hasher.input(token_bytes);
            let hashed_token = hasher.result_str();

            println!("{:?}", hashed_token);

            let based = base64::encode(hashed_token);
            println!("{:?}", based);
            println!("upgrade");
            let status = "HTTP/1.1 101 Switching Protocols";
            let header = format!(
                "{}{}{:?}{}",
                status,
                "Upgrade: websocket
            Connection: Upgrade
            Sec-WebSocket-Accept: ",
            based,
                "
            Sec-WebSocket-Protocol: chat
                "
            );
            println!("{}", header);
            let res = header + "\r\n";
            let data = res.as_bytes();
            println!("{:?}", data);
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
            let data = res.as_bytes();
            stream.write(data);
        }
        None => {}
    }
}
