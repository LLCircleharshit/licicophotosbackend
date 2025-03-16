use crate::models::ResponseData;

fn simple_decrypt(data: &[u8], key: &str) -> String {
    data.iter()
        .zip(key.as_bytes().iter().cycle())
        .map(|(&c, &k)| (c ^ k) as char)
        .collect()
}

#[allow(dead_code)]
fn base64_decode(data: &str) -> Vec<u8> {
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
    decoded
}

#[allow(unused_variables)]
pub fn handle_auth(username: String, password: String, operation: String) -> ResponseData {
    // Decode Base64-encoded encrypted username and password
    let decoded_username = base64_decode(&username);
    let decoded_password = base64_decode(&password);

    // Decrypt the decoded bytes
    let decrypted_username = simple_decrypt(&decoded_username, "secretkey");
    let decrypted_password = simple_decrypt(&decoded_password, "secretkey");
    
    // println!("Decrypted Username: {}", decrypted_username);
    // println!("Decrypted Password: {}", decrypted_password);

    ResponseData {
        username: decrypted_username,
        password: decrypted_password,
        message: "dec successful".to_string(),
    }
}
