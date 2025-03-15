use std::collections::HashMap;

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String,String>,
    pub params: HashMap<String,String>,
    pub body: Vec<u8>,
    pub extensions: HashMap<String, Box<dyn std::any::Any + Send + 'static>>
}

impl Request {
    pub fn new(method: String, path: String) -> Self{
        Request{
            method,
            path,
            headers: HashMap::new(),
            params: HashMap::new(),
            body: Vec::new(),
            extensions: HashMap::new()
        }
    }
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

    pub fn html(content: &str) -> Self {
        let mut response = Response::new();
        response.headers.insert("Content-Type".to_string(), "text/html".to_string());
        response.body = content.as_bytes().to_vec();
        response
    }

    pub fn json(content: String) -> Self {
        let mut response = Response::new();
        response.headers.insert("Content-Type".to_string(), "application/json".to_string());
        response.body = content.as_bytes().to_vec();
        response
    }

    pub fn not_found() -> Self {
        let mut response = Response::new();
        response.status = 404;
        response.body = b"404 Not Found".to_vec();
        response
    }

    pub fn unauthorized() -> Self {
        let mut response = Response::new();
        response.status = 401;
        response.body = b"401 Unauthorized".to_vec();
        response
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}
