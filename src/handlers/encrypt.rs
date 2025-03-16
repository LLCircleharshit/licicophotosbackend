// handlers/encrypt.rs
use crate::models::ResponseData;

#[allow(dead_code)]
fn simple_encrypt(data: &str, key: &str) -> Vec<u8> {
    data.as_bytes()
        .iter()
        .zip(key.as_bytes().iter().cycle())
        .map(|(&c, &k)| c ^ k)  
        .collect()
}


#[allow(dead_code)]
fn simple_decrypt(data: &[u8], key: &str) -> String {
    data.iter()
        .zip(key.as_bytes().iter().cycle())
        .map(|(&c, &k)| (c ^ k) as char)
        .collect()
}


#[allow(dead_code)]
#[allow(unused_variables)]
fn base64_encode(data: &str) -> String {
    let bytes = data.as_bytes();
    let mut encoded = String::new();
    const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    for chunk in bytes.chunks(3) {
        let mut buffer = 0u32;
        let mut padding = 0;

        for (i, &byte) in chunk.iter().enumerate() {
            buffer |= (byte as u32) << (16 - i * 8);
        }

        for i in 0..4 {
            if i * 6 > chunk.len() * 8 {
                encoded.push('=');
                padding += 1;
            } else {
                let index = ((buffer >> (18 - i * 6)) & 0x3F) as usize;
                encoded.push(BASE64_TABLE[index] as char);
            }
        }
    }
    encoded
}

#[allow(dead_code)]
fn base64_decode(data: &str) -> String {
    let mut decoded = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    for c in data.chars() {
        if c == '=' {
            break;
        }
        if let Some(index) = BASE64_TABLE.iter().position(|&b| b == c as u8) {
            buffer = (buffer << 6) | (index as u32);
            bits += 6;

            if bits >= 8 {
                bits -= 8;
                decoded.push((buffer >> bits) as u8);
                buffer &= (1 << bits) - 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}


#[allow(unused_variables)]
    pub fn handle_auth(username: String, password: String, operation: String) -> ResponseData {
        let encrypted_username = simple_encrypt(&username, "secretkey");
        let encrypted_password = simple_encrypt(&password, "secretkey");
    
        let encoded_username = base64_encode(&String::from_utf8_lossy(&encrypted_username));
        let encoded_password = base64_encode(&String::from_utf8_lossy(&encrypted_password));
    
        ResponseData {
            username: encoded_username,
            password: encoded_password,
            message: "Enc successful".to_string(),
        }
    }
    