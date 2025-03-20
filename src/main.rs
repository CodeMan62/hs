mod thread_pool;
mod types;
mod route;

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream};
use thread_pool::ThreadPool;
use route::{Router, parse_request, response_to_string};
use types::Response;
use std::sync::Arc;

fn handle_client(mut stream: TcpStream, router: Arc<Router>) {
    let buf_reader = BufReader::new(&stream);
    let request_lines: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let response = match parse_request(&request_lines) {
        Some(mut request) => {
            println!("Request: {} {}", request.method, request.path);
            println!("Headers:");
            for (key, value) in &request.headers {
                println!("  {}: {}", key, value);
            }

            // Route the request
            router.route(&mut request).unwrap_or_else(|| {
                // Return 404 Not Found if no route matches
                Response::not_found()
            })
        },
        None => Response::new().with_status(400) // Bad Request if parsing fails
    };

    // Convert response to HTTP string and send
    let response_string = response_to_string(&response);
    stream.write_all(response_string.as_bytes()).unwrap();
}

async fn handle_client_async(mut stream: TokioTcpStream, router: Arc<Router>) {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut request_lines = Vec::new();
    let mut line = String::new();

    loop {
        line.clear();
        if buf_reader.read_line(&mut line).await.unwrap() == 0 {
            break;
        }
        if line == "\r\n" {
            break;
        }
        request_lines.push(line.trim().to_string());
    }

    let response = match parse_request(&request_lines) {
        Some(mut request) => {
            println!("Async Request: {} {}", request.method, request.path);

            // Route the request
            router.route(&mut request).unwrap_or_else(|| {
                // Return 404 Not Found if no route matches
                Response::not_found()
            })
        },
        None => Response::new().with_status(400) // Bad Request if parsing fails
    };

    let response_string = response_to_string(&response);
    writer.write_all(response_string.as_bytes()).await.unwrap();
}

fn create_router() -> Router {
    let mut router = Router::new();

    router.set_static_dir("./public");

    router.serve_static("/static", "./public");

    // Add a route for the home page
    router.get("/", |_req| {
        Response::html("<html><body><h1>Welcome to Rust HTTP Server!</h1><p>Home page</p></body></html>")
    });

    // Route with dynamic parameter
    router.get("/user/:id", |req| {
        let user_id = req.params.get("id").unwrap_or(&"unknown".to_string()).clone();
        Response::html(&format!(
            "<html><body><h1>User Profile</h1><p>User ID: {}</p></body></html>",
            user_id
        ))
    });

    // API route example
    router.get("/api/status", |_req| {
        Response::json("{\"status\":\"online\",\"version\":\"1.0\"}".to_string())
    });

    // Post example
    router.post("/api/data", |req| {
        // In a real application, you would parse the body here
        Response::json("{\"success\":true,\"message\":\"Data received\"}".to_string())
    });

    router
}

fn run_threaded_server(router: Arc<Router>) {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);
    println!("Threaded server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let router_clone = Arc::clone(&router);
                pool.execute(move || {
                    handle_client(stream, router_clone);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn run_async_server(router: Arc<Router>) {
    let listener = TokioTcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("Async server listening on port 8081");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("New connection: {}", addr);
                let router_clone = Arc::clone(&router);
                tokio::spawn(async move {
                    handle_client_async(stream, router_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a router with our routes
    let router = Arc::new(create_router());

    // Clone the router for the threaded server
    let threaded_router = Arc::clone(&router);

    // Spawn the threaded server in a separate thread
    std::thread::spawn(move || {
        run_threaded_server(threaded_router);
    });

    // Run the async server in the main thread
    run_async_server(router).await;
}

// Deprecated functions that we're replacing with our new routing system
#[allow(dead_code)]
fn create_response(request_lines: &[String]) -> String {
    if let Some(request_line) = request_lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            return format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: text/html\r\n\
                Content-Length: 98\r\n\
                \r\n\
                <html><body><h1>Hello from Rust HTTP Server!</h1><p>You requested: {} {}</p></body></html>",
                method, path
            );
        }
    }

    "HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
}

#[allow(dead_code)]
fn handle_request(request_lines: Vec<String>, stream: &mut TcpStream) {
    if let Some(request_line) = request_lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            println!("Request: {} {}", method, path);
            println!("Headers:");
            for header in &request_lines[1..] {
                println!("  {}", header);
            }

            let response = create_response(&request_lines);
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}
