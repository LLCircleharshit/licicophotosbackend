mod models;
mod handlers;
use models::{RequestData, ResponseData};
#[allow(unused_imports)]
use handlers::{encrypt, decrypt};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = Vec::new();
    let mut temp_buf = [0; 1024];

    // Read the incoming stream in chunks
    loop {
        match stream.read(&mut temp_buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                buffer.extend_from_slice(&temp_buf[..n]);
                // Check if the request has ended (CRLF CRLF)
                if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read stream: {}", e);
                return;
            }
        }
    }

    let request = String::from_utf8_lossy(&buffer);
    println!("Full request: {}", request);

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
        if let Some(index) = request.find("\r\n\r\n") {
            let json_body = &request[index + 4..].trim();
            println!("Extracted JSON body: {}", json_body);

            let request_data: Result<RequestData, _> = serde_json::from_str(json_body);
            let response_message = match request_data {
                Ok(RequestData::Auth { username, password, operation }) => {
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

            stream.write_all(response.as_bytes()).unwrap();
        } else {
            let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server listening on 0.0.0.0:8080");

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
