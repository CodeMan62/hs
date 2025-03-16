use std::collections::HashMap;

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.body = body.as_bytes().to_vec();
        self
    }

    pub fn with_body_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn html(content: &str) -> Self {
        Response::new()
            .with_header("Content-Type", "text/html")
            .with_body(content)
    }

    pub fn text(content: &str) -> Self {
        Response::new()
            .with_header("Content-Type", "text/plain")
            .with_body(content)
    }

    pub fn json(content: String) -> Self {
        Response::new()
            .with_header("Content-Type", "application/json")
            .with_body(&content)
    }

    pub fn not_found() -> Self {
        Response::new()
            .with_status(404)
            .with_header("Content-Type", "text/html")
            .with_body("<html><body><h1>404 Not Found</h1><p>The requested resource could not be found.</p></body></html>")
    }
}
