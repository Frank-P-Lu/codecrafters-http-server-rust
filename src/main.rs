use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::{env, fs, thread};
use request::HttpMethod;
use crate::response::Response;

mod request;
mod response;

fn handle(stream: &mut TcpStream, directory: Option<String>) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];

    let read_result = stream.read(&mut buffer)?;
    let result_string = std::str::from_utf8(&buffer[..read_result])?;

    println!("Request: {}", result_string);
    let request = request::parse_request(result_string);
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
                    response::resp200(Response {
                        status_code: 200,
                        content: params[0].to_string(),
                        content_type: None,
                        encoding: request.encoding,
                    })
                }
                "user-agent" => {
                    println!("User-Agent: {:?}", request.user_agent);
                    match request.user_agent {
                        Some(ua) => {
                            response::resp200(
                                Response {
                                    status_code: 200,
                                    content: ua.to_string(),
                                    content_type: None,
                                    encoding: None,
                                })
                        }
                        None => {
                            response::resp404()
                        }
                    }
                }
                "files" => {
                    let file_path = format!("{}/{}", directory.unwrap(), params[0]);
                    match request.http_method {
                        HttpMethod::GET => {
                            match fs::read_to_string(file_path) {
                                Ok(file_contents) => {
                                    response::resp200(
                                        Response {
                                            status_code: 200,
                                            content: file_contents.to_string(),
                                            content_type: Some("application/octet-stream".to_string()),
                                            encoding: None,
                                        }
                                    )
                                }
                                Err(err) => {
                                    println!("Error reading file: {}", err);
                                    response::resp404()
                                }
                            }
                        }
                        HttpMethod::POST => {
                            match fs::write(file_path, request.body.unwrap()) {
                                Ok(_) => {
                                    response::resp201()
                                }
                                Err(err) => {
                                    println!("Error writing file: {}", err);
                                    response::resp404()
                                }
                            }
                        }
                    }
                }
                _ => { response::resp404() }
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
