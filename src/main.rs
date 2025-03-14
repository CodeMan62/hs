mod thread_pool;

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream};
use thread_pool::ThreadPool;

// Synchronous request handler
fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_lines: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    handle_request(request_lines, &mut stream);
}

// Asynchronous request handler
async fn handle_client_async(mut stream: TokioTcpStream) {
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

    let response = create_response(&request_lines);
    writer.write_all(response.as_bytes()).await.unwrap();
}

// Common response creation logic
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

// Common request handling logic
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

// Threaded server implementation
fn run_threaded_server() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);
    println!("Threaded server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                pool.execute(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

// Async server implementation
async fn run_async_server() {
    let listener = TokioTcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("Async server listening on port 8081");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("New connection: {}", addr);
                tokio::spawn(async move {
                    handle_client_async(stream).await;
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
    // Spawn the threaded server in a separate thread
    std::thread::spawn(|| {
        run_threaded_server();
    });

    // Run the async server in the main thread
    run_async_server().await;
}
