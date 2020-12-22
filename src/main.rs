extern crate base64;
extern crate hex;
extern crate httparse;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    // 接続ごとにthreadを作り、その中で処理を行う
    for stream in listener.incoming() {
        println!("======== ffff ffff =========");
        thread::spawn(move || {
            let s = stream.unwrap();
            handle_client(&s);
        });
    }
}

fn handle_client(mut stream: &TcpStream) {
    let key = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let client_code = "function () {
  var ws = new WebSocket('ws://localhost:8080/websocket', ['test', 'chat']);
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
    req.parse(&buf).unwrap();
    let path = req.path.expect("fail");
    match path {
        "/" => {
            let body =
                "<html><head><title>rust web socket</title><script type='text/javascript'>("
                    .to_string()
                    + client_code
                    + ")()</script></head><body>hello world!!!!!</body></html>";
            let status = "HTTP/1.1 200 OK\r\n".to_string();
            let header = status + "Content-Type: text/html; charset=UTF-8\r\n\r\n";
            let res = header + &body + "\r\n";

            let data = res.as_bytes();
            stream.write(data).unwrap();
        }
        "/websocket" => {
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
            stream.write(data).unwrap();

            loop {
                let mut msg_buf = [0; 1024];
                if stream.read(&mut msg_buf).is_ok() {
                    let opcode = msg_buf[0] & 0x0f;
                    if opcode == 1 {
                        let payload_length = (msg_buf[1] & 0b1111110 ) as usize;
                        let mask: Vec<u8> = msg_buf[2..=5].to_vec();

                        let mut payload = Vec::<u8>::with_capacity(payload_length);
                        for i in 0..payload_length {
                            payload.push(msg_buf[6 + i] ^ mask[i % 4]);
                        }
                        let payload = String::from_utf8(payload).unwrap();
                        println!("{}", payload);
                        stream.write(&[129, 5, 72, 101, 108, 108, 111]).unwrap();
                    }
                } else {
                    break;
                }
            }
        }
        _ => {}
    }
}
