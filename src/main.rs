use std::collections::HashMap;
use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn resp200(content: &str) -> String {
    let content_len = content.len();
    return format!(
        "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/plain\r\n\
                    Content-Length: {content_len}\r\n\r\n\
                    {content}"
    ).to_string();
}

#[derive(Debug)]
enum HttpMethod {
    GET,
}

#[derive(Debug)]
struct Request {
    http_method: HttpMethod,
    path: String,
    user_agent: Option<String>,
}

fn parse_request(request_string: &str) -> Request {
    // Typical request format
    // GET /index.html HTTP/1.1
    // Host: localhost:4221
    // User-Agent: curl/7.64.1

    let lines: Vec<&str> = request_string.split("\r\n")
        .filter(|s| !s.is_empty())
        .collect();
    let m: HashMap<&str, &str> = lines[1..].iter().map(|line|
        line.trim().split(": ").collect()
    ).filter(
        |line_parts : &Vec<&str>| {
            debug_assert!(line_parts.len() == 2, "Invalid request line: {}", line_parts[0]);
            return line_parts.len() == 2
        }
    ).map(
        |line_parts|
            (line_parts[0].trim(), line_parts[1].trim())
    ).collect();

    let request_line: Vec<&str> = lines[0].split(" ").collect();
    let path = request_line[1];
    let user_agent = match m.get("User-Agent") {
        Some(ua) => Some(ua.to_string()),
        None => None,
    };
    return Request {
        http_method: HttpMethod::GET,
        path: path.to_string(),
        user_agent,
    };
}

fn process(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];

    let read_result = stream.read(&mut buffer)?;
    let result_string = std::str::from_utf8(&buffer[..read_result])?;

    println!("Request: {}", result_string);
    let request = parse_request(result_string);
    println!("Parsed request: {:?}", request);
    let response = match request.path.as_str() {
        "/" => {
            "HTTP/1.1 200 OK\r\n\r\n".to_string()
        }
        _ => {
            let path_segments: Vec<&str> = request.path.split("/").collect();
            let path_start = path_segments[1];
            println!("Path start {}", path_start);
            match path_start {
                "echo" => {
                    let param = path_segments[2];
                    resp200(param)
                }
                "user-agent" => {
                    println!("User-Agent: {:?}", request.user_agent);
                    match request.user_agent {
                        Some(ua) => {
                            resp200(ua.as_str())
                        }
                        None => {
                            "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
                        }
                    }
                }
                _ => { "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string() }
            }
            // "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
        }
    };
    stream.write_all(response.as_bytes()).unwrap();
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    // Debug
    // let listener = TcpListener::bind("127.0.0.1:4222").unwrap();

    for stream in listener.incoming() {
        let mut new_stream = stream.unwrap();
        thread::spawn(move || {
            let _ = process(&mut new_stream);
        });
        println!("accepted new connection");
    }
}
