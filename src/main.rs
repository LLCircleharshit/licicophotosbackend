mod models;
mod handlers;
use models::{RequestData, ResponseData};
#[allow(unused_imports)]
use handlers::{encrypt, decrypt};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(size) if size > 0 => size,
        _ => return,
    };

    let request: String = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    // println!("Full request:\n{}", request);

    let request_line = request.lines().next().unwrap_or("");

    // Handle OPTIONS preflight
    if request_line.starts_with("OPTIONS /") {
        let response = "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Content-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    // Handle POST request
    if request_line.starts_with("POST /") {
        // Extract Content-Length
        let content_length = request
            .lines()
            .find(|line| line.to_ascii_lowercase().starts_with("content-length:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|len| len.trim().parse::<usize>().ok())
            .unwrap_or(0);
        // println!("Content-Length: {}", content_length);

        // Find the start of the body
        let body_start = request.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let mut body = String::from(&request[body_start..]);

        // If the body is incomplete, read more data
        while body.len() < content_length {
            let mut extra_buffer = [0; 4096];
            match stream.read(&mut extra_buffer) {
                Ok(size) if size > 0 => {
                    body.push_str(&String::from_utf8_lossy(&extra_buffer[..size]));
                }
                _ => break,
            }
        }

        // println!("Extracted JSON body: '{}'", body);

        // Try to parse the JSON body
        let request_data: Result<RequestData, _> = serde_json::from_str(&body);
        let response_message = match request_data {
            Ok(RequestData::Auth { username, password, operation }) => {
                // println!("Parsed JSON: username='{}', password='{}', operation='{}'", username, password, operation);
                match operation.as_str() {
                    "encrypt" => encrypt::handle_auth(username, password, operation),
                    "decrypt" => decrypt::handle_auth(username, password, operation),
                    _ => ResponseData {
                        message: "Unknown operation".to_string(),
                        username: "".to_string(),
                        password: "".to_string(),
                    },
                }
            }
            Err(e) => {
                println!("JSON parsing error: {:?}", e);
                ResponseData {
                    message: "Invalid JSON format".to_string(),
                    username: "".to_string(),
                    password: "".to_string(),
                }
            }
        };

        let json_response = serde_json::to_string(&response_message).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\r\n\
            {}",
            json_response.len(),
            json_response
        );

        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to send response: {}", e);
        }
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to send 404 response: {}", e);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    // println!("Server listening on 0.0.0.0:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
