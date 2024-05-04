use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn process(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    let mut buffer = [0; 1024];

    let read_result = stream.read(&mut buffer)?;
    let result_string = std::str::from_utf8(&buffer[..read_result])?;

    // Typical request format
    // GET /index.html HTTP/1.1
    // Host: localhost:4221
    // User-Agent: curl/7.64.1
    let parts: Vec<&str> = result_string.split("\r\n").collect();
    let first_line: Vec<&str> = parts[0].split(" ").collect();
    let path = first_line[1];
    match path {
        "/" => {
            Ok("HTTP/1.1 200 OK\r\n\r\n".to_string())
        }
        _ => {
            let path_start : Vec<&str> = path.split("/").collect();
            if path_start[1] == "echo" {
                let param = path_start[2];
                let param_len = param.len();
                return Ok(
                    format!(
                    "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/plain\r\n\
                    Content-Length: {param_len}\r\n\r\n\
                    {param}"
                    ).to_string());
            }
            Ok("HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string())
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let mut new_stream = stream.unwrap();
        if let Ok(response) = process(&mut new_stream) {
            new_stream.write_all(response.as_bytes()).unwrap()
        }
        println!("accepted new connection");
    }
}
