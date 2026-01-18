use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

fn main() -> Result<(), String> {
    let server_addr = std::env::args().nth(1).ok_or(
        "Server bind address missing. Expected: ./client SERVER_ADDRESS:SERVER_PORT CLIENT_ADDRESS:CLIENT_PORT 32BYTE_TOKEN".to_string(),
    )?;
    let client_addr = std::env::args().nth(2).ok_or(
        "Client bind address missing. Expected: ./client SERVER_ADDRESS:SERVER_PORT CLIENT_ADDRESS:CLIENT_PORT 32BYTE_TOKEN".to_string(),
    )?;
    let token = std::env::args().nth(3).ok_or(
        "Token missing. Expected: ./client SERVER_ADDRESS:SERVER_PORT CLIENT_ADDRESS:CLIENT_PORT 32BYTE_TOKEN".to_string(),
    )?;

    let listener = UdpSocket::bind(&client_addr)
        .map_err(|error| format!("Failed to bind to {client_addr}: {error}.",))?;

    listener
        .send_to(token.as_bytes(), &server_addr)
        .map_err(|error| format!("Failed to send to {server_addr}: {error}.",))?;

    let mut socket_addr_data = [0u8; 50];
    let received_len = listener
        .recv(&mut socket_addr_data)
        .map_err(|error| format!("Failed to receive from {server_addr}: {error}."))?;

    let required_len = socket_addr_data[0] as usize;

    if received_len - 1 != required_len {
        return Err(format!(
            "Expected to receive {required_len} socket addr bytes, but received {} socket addr bytes.",
            received_len - 1
        ));
    }

    let socket_addr_str = str::from_utf8(&socket_addr_data[1..received_len]).map_err(|error| {
        format!(
            "Failed to parse socket addr {:?}: {error}.",
            &socket_addr_data[1..received_len]
        )
    })?;
    let socket_addr = SocketAddr::from_str(socket_addr_str)
        .map_err(|error| format!("Failed to parse socket addr {socket_addr_str}: {error}."))?;

    println!("Received peer address: {socket_addr}");

    Ok(())
}
