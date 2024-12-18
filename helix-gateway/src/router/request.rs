use std::{collections::HashMap, io::Read, net::TcpStream};

pub struct Request {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub path: String,
    pub body: Vec<u8>,
}

impl Request {
    pub fn from_stream<R: Read>(stream: &mut R) -> std::io::Result<Request> {
        let mut buf = [0; 4096];
        // consider using &str to avoid heap allocation
        let mut request_data = String::new();

        // read data
        loop {
            let bytes = stream.read(&mut buf).unwrap();
            request_data.push_str(&String::from_utf8_lossy(&buf[..bytes]));

            if request_data.contains("\r\n\r\n") || bytes == 0 {
                break;
            }
        }

        // TODO: read and split up data
        let mut lines = request_data.lines();
        let first_line = lines.next().unwrap_or("");
        let mut parts = first_line.split_whitespace();

        let method = parts.next().unwrap_or("GET").to_string();
        let path = parts.next().unwrap_or("/").to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }

        let mut body = Vec::new();
        if let Some(content_length) = headers.get("Content-Length") {
            if let Ok(length) = content_length.parse::<usize>() {
                let mut buffer = vec![0; length];
                stream.read_exact(&mut buffer)?;
                body = buffer;
            }
        }

        // construct request object
        Ok(Request {
            method,
            headers,
            path,
            body,
        })
    }
}
