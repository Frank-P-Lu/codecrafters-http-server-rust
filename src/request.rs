use std::collections::HashMap;

pub fn parse_request(request_string: &str) -> Request {
    // Typical request format
    // GET /index.html HTTP/1.1
    // Host: localhost:4221
    // User-Agent: curl/7.64.1
    // Accept-Encoding: gzip

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
    let encoding = m.get("Accept-Encoding").and_then(|&ua| {
        if ua == "gzip" {
            Some(Encoding::Gzip)
        } else {
            None
        }
    });
    let body = match http_method {
        HttpMethod::GET => None,
        HttpMethod::POST => match m.get("Content-Length") {
            Some(content_length) => {
                Some(lines.last().unwrap().to_string())
            }
            None => None,
        },
    };
    return Request {
        http_method,
        path: path.to_string(),
        user_agent,
        encoding,
        body,
    };
}


#[derive(Debug)]
pub enum Encoding {
    Gzip,
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
}

#[derive(Debug)]
pub struct Request {
    pub http_method: HttpMethod,
    pub path: String,
    pub user_agent: Option<String>,
    pub body: Option<String>,
    pub encoding: Option<Encoding>,
}
