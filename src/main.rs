use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_lines: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // Parse the request
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

            // Send a response
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: text/html\r\n\
                Content-Length: 98\r\n\
                \r\n\
                <html><body><h1>Hello from Rust HTTP Server!</h1><p>You requested: {} {}</p></body></html>",
                method, path
            );

            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                handle_client(stream);
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
