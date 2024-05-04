use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn resp200(content: &str) -> String {
    let content_len = content.len();
    return format!(
        "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/plain\r\n\
                    Content-Length: {content_len}\r\n\r\n\
                    {content}"
    ).to_string();
}

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
    let user_agent_line = parts[2];
    let user_agent: &str = user_agent_line.split(" ").collect::<Vec<&str>>()[1];
    match path {
        "/" => {
            Ok("HTTP/1.1 200 OK\r\n\r\n".to_string())
        }
        _ => {
            let path_segments: Vec<&str> = path.split("/").collect();
            let path_start = path_segments[1];
            return match path_start {
                "echo" => {
                    let param = path_segments[2];
                    return Ok(resp200(param));
                }
                "user-agent" => {
                    return Ok(
                        resp200(user_agent));
                }
                _ => { Ok("HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()) }
            };
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let mut new_stream = stream.unwrap();
        if let Ok(response) = process(&mut new_stream) {
            new_stream.write_all(response.as_bytes()).unwrap()
        }
        println!("accepted new connection");
    }
}
