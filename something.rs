use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use crate::types::{Request, Response};

enum PathSegment {
    Static(String),
    Param(String),
}

struct Route {
    segments: Vec<PathSegment>,
    handler: Handler,
}

pub type Handler = Arc<dyn Fn(&mut Request) -> Response + Send + Sync>;

impl Route {
    fn new<F>(pattern: &str, handler: F) -> Self
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        Route {
            segments: parse_path_pattern(pattern),
            handler: Arc::new(handler),
        }
    }

    fn matches(&self, path: &str, params: &mut HashMap<String, String>) -> bool {
        let path_segments: Vec<&str> = path.split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // If segment count doesn't match, route doesn't match
        if path_segments.len() != self.segments.len() {
            return false;
        }

        // Check if segments match
        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::Static(s) => {
                    if s != path_segments[i] {
                        return false;
                    }
                },
                PathSegment::Param(name) => {
                    // For parameter segments, extract and store the value
                    params.insert(name.clone(), path_segments[i].to_string());
                }
            }
        }

        true
    }
}


fn parse_path_pattern(pattern: &str) -> Vec<PathSegment>{
    pattern.split("/")
    .filter(|s| !s.is_empty())
    .map(|segment| {
        if segment.starts_with(":") {
            PathSegment::Param(segment[1..].to_string())
        }else {
            PathSegment::Static(segment.to_string())
        }
    })
    .collect()
}

pub struct Router {
    routes: HashMap<String, Vec<Route>>,
    static_dir: Option<PathBuf>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            static_dir: None,
        }
    }

    pub fn set_static_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.static_dir = Some(PathBuf::from(dir.as_ref()));
        self
    }

    pub fn add_route<F>(&mut self, method: &str, path: &str, handler: F)
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        let method = method.to_uppercase();
        let routes = self.routes.entry(method).or_insert_with(Vec::new);
        routes.push(Route::new(path, handler));
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        self.add_route("GET", path, handler);
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        self.add_route("POST", path, handler);
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        self.add_route("PUT", path, handler);
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request) -> Response + Send + Sync + 'static,
    {
        self.add_route("DELETE", path, handler);
    }

    pub fn serve_static(&mut self, url_path: &str, dir_path: &str) -> &mut Self {
        let static_dir = PathBuf::from(dir_path);
        self.static_dir = Some(static_dir.clone());

        // Add a route handler for the static file URL path
        self.get(&format!("{}/*filepath", url_path.trim_end_matches('/')), move |req| {
            // Extract the file path from the URL
            if let Some(file_path_str) = req.params.get("filepath") {
                let mut file_path = static_dir.clone();
                // Use only the filepath parameter, not the entire path
                for segment in file_path_str.split('/') {
                    // Prevent directory traversal attacks
                    if segment != ".." && segment != "." && !segment.is_empty() {
                        file_path.push(segment);
                    }
                }

                // Try to read the file
                match fs::read(&file_path) {
                    Ok(content) => {
                        // Determine content type from file extension
                        let content_type = get_content_type(&file_path);
                        Response::new()
                            .with_header("Content-Type", content_type)
                            .with_body_bytes(content)
                    },
                    Err(e) => {
                        println!("Error reading file {:?}: {}", file_path, e);
                        match e.kind() {
                            std::io::ErrorKind::NotFound => Response::not_found(),
                            std::io::ErrorKind::PermissionDenied => {
                                Response::new()
                                    .with_status(403)
                                    .with_body("403 Forbidden: Access to this resource is denied")
                            },
                            _ => Response::new()
                                .with_status(500)
                                .with_body("500 Internal Server Error")
                        }
                    }
                }
            } else {
                Response::not_found()
            }
        });

        self
    }

    pub fn route(&self, request: &mut Request) -> Option<Response> {
        // First try to match defined routes
        if let Some(routes) = self.routes.get(&request.method) {
            for route in routes {
                // Try to match the route
                if route.matches(&request.path, &mut request.params) {
                    // Route matched, call the handler
                    return Some((route.handler)(request));
                }
            }
        }

        // If no route matched and it's a GET request, try to serve a static file
        if request.method == "GET" && self.static_dir.is_some() {
            let path = request.path.trim_start_matches('/');
            let mut file_path = self.static_dir.as_ref().unwrap().clone();

            // Build the file path, preventing directory traversal
            for segment in path.split('/') {
                if segment != ".." && segment != "." && !segment.is_empty() {
                    file_path.push(segment);
                }
            }

            // Try to read the file
            match fs::read(&file_path) {
                Ok(content) => {
                    // Determine content type from file extension
                    let content_type = get_content_type(&file_path);
                    return Some(Response::new()
                        .with_header("Content-Type", content_type)
                        .with_body_bytes(content));
                },
                Err(_) => {
                    // File not found or other error, continue to return None
                }
            }
        }

        // No route matched
        None
    }
}

fn get_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("xml") => "application/xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",  // Default binary data
    }
}

pub fn parse_request(request_lines: &[String]) -> Option<Request> {
    if request_lines.is_empty() {
        return None;
    }

    let request_line = &request_lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();

    let mut headers = HashMap::new();
    for line in &request_lines[1..] {
        if let Some(pos) = line.find(':') {
            let (key, value) = line.split_at(pos);
            let value = value[1..].trim();
            headers.insert(key.to_string(), value.to_string());
        }
    }

    Some(Request {
        method,
        path,
        headers,
        body: Vec::new(),  // We'll parse body later if needed
        params: HashMap::new(),
    })
}

pub fn response_to_string(response: &Response) -> String {
    let status_text = match response.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let mut response_string = format!("HTTP/1.1 {} {}\r\n", response.status, status_text);

    // Add Content-Length header if not already present
    if !response.headers.contains_key("Content-Length") {
        response_string.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    }

    // Add all headers
    for (key, value) in &response.headers {
        response_string.push_str(&format!("{}: {}\r\n", key, value));
    }

    // Add empty line to separate headers from body
    response_string.push_str("\r\n");

    // We need to convert the response to a byte array for sending
    // For simplicity in this example, we'll convert binary data to a string
    // In a real implementation, you would keep this as binary data
    let mut result = response_string.into_bytes();
    result.extend_from_slice(&response.body);

    // This is a hack for the current implementation
    // In a real implementation, you'd handle binary data properly
    String::from_utf8_lossy(&result).to_string()
}
