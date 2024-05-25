use crate::request::Encoding;

pub struct Response {
    pub status_code: u16,
    pub content: String,
    pub content_type: Option<String>,
    pub encoding: Option<Encoding>
}

pub fn resp200(r: Response) -> String {
    let content_type = r.content_type.unwrap_or_else(|| "text/plain".to_string());
    let content_len = r.content.len();
    let encoding = match r.encoding {
        Some(Encoding::Gzip) => {
            "Content-Encoding: gzip\r\n".to_string()
        }
        None => "".to_string()
    };
    println!("{} {:?}", encoding.to_string(), r.encoding);
    let content = r.content;
    return format!(
        "HTTP/1.1 200 OK\r\n\
        {encoding}\
        Content-Type: {content_type}\r\n\
        Content-Length: {content_len}\r\n\
        \r\n\
        {content}"
    ).to_string();
}

pub fn resp201() -> String {
    return "HTTP/1.1 201 Created\r\n\r\n".to_string();
}

pub fn resp404() -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}
