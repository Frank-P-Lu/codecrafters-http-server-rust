use std::collections::HashMap;
use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::{env, fs, thread};

fn resp201() -> String {
    return "HTTP/1.1 201 OK\r\n\r\n".to_string();
}

fn resp200(content: &str, content_type: Option<&str>) -> String {
    let content_type = content_type.unwrap_or("text/plain");
    let content_len = content.len();
    return format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: {content_type}\r\n\
        Content-Length: {content_len}\r\n\
        \r\n\
        {content}"
    ).to_string();
}

fn resp404() -> String {
    "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
}

#[derive(Debug)]
enum HttpMethod {
    GET,
    POST,
}

#[derive(Debug)]
struct Request {
    http_method: HttpMethod,
    path: String,
    user_agent: Option<String>,
    body: Option<String>,
}

fn parse_request(request_string: &str) -> Request {
    // Typical request format
    // GET /index.html HTTP/1.1
    // Host: localhost:4221
    // User-Agent: curl/7.64.1

    let lines: Vec<&str> = request_string.split("\r\n")
        .filter(|s| !s.is_empty())
        .collect();
    let m: HashMap<&str, &str> = lines[1..].iter().map(
        |line|
            line.trim().split(": ").collect())
        .filter(
            |line_parts: &Vec<&str>| {
                debug_assert!(line_parts.len() == 2, "Invalid request line: {}", line_parts[0]);
                return line_parts.len() == 2;
            }
        ).map(
        |line_parts|
            (line_parts[0].trim(), line_parts[1].trim())
    ).collect();

    let request_line: Vec<&str> = lines[0].split(" ").collect();
    let http_method = match request_line[0] {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        _ => panic!("Invalid HTTP method: {}", request_line[0]),
    };
    let path = request_line[1];


    let user_agent = match m.get("User-Agent") {
        Some(ua) => Some(ua.to_string()),
        None => None,
    };
    let body = match http_method {
        HttpMethod::GET => None,
        HttpMethod::POST => match m.get("Content-Length") {
            Some(content_length) => {
                // let content_length = content_length.parse::<usize>()?;
                // let mut body = String::new();
                Some(lines.last().unwrap().to_string())
            }
            None => None,
        },
    };
    return Request {
        http_method,
        path: path.to_string(),
        user_agent,
        body,
    };
}

fn handle(stream: &mut TcpStream, directory: Option<String>) -> Result<(), Box<dyn Error>> {
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
            let params = &path_segments[2..];
            println!("Path start {}", path_start);
            match path_start {
                "echo" => {
                    resp200(params[0], None)
                }
                "user-agent" => {
                    println!("User-Agent: {:?}", request.user_agent);
                    match request.user_agent {
                        Some(ua) => {
                            resp200(ua.as_str(), None)
                        }
                        None => {
                            resp404()
                        }
                    }
                }
                "files" => {
                    let file_path = format!("{}/{}", directory.unwrap(), params[0]);
                    match request.http_method {
                        HttpMethod::GET => {
                            match fs::read_to_string(file_path) {
                                Ok(file_contents) => {
                                    resp200(file_contents.as_str(), Some("application/octet-stream"))
                                }
                                Err(err) => {
                                    println!("Error reading file: {}", err);
                                    resp404()
                                }
                            }
                        }
                        HttpMethod::POST => {
                            match fs::write(file_path, request.body.unwrap()) {
                                Ok(_) => {
                                    resp201()
                                }
                                Err(err) => {
                                    println!("Error writing file: {}", err);
                                    resp404()
                                }
                            }
                        }
                    }
                }
                _ => { resp404() }
            }
        }
    };
    stream.write_all(response.as_bytes()).unwrap();
    Ok(())
}

fn main() {
    println!("Starting server!");
    let args: Vec<String> = env::args().collect();

    // Assume for now that directory is the third argument
    let directory = match args.len() {
        3 => Some(&args[2]),
        _ => None,
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    // Debug
    // let listener = TcpListener::bind("127.0.0.1:4222").unwrap();

    for stream in listener.incoming() {
        let mut new_stream = stream.unwrap();
        let cloned_directory: Option<String> = directory.cloned();
        thread::spawn(move || {
            let _ = handle(&mut new_stream, cloned_directory);
        });
        println!("accepted new connection");
    }
}
