use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

const HELLO_MESSAGE: &[u8] = b"HELLO";

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
    let other_peer = SocketAddr::from_str(socket_addr_str)
        .map_err(|error| format!("Failed to parse socket addr {socket_addr_str}: {error}."))?;

    let listener = Arc::new(listener);

    establish_connection(listener, other_peer)?;

    println!("Received peer address and successfuly established connection: {other_peer}");

    Ok(())
}

fn establish_connection(listener: Arc<UdpSocket>, other_peer: SocketAddr) -> Result<(), String> {
    let (ping_sender, ping_handle) = spawn_hello_ping(Arc::clone(&listener), other_peer);

    listener
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    let mut hello_msg = [0u8; 50];
    for _ in 0..10 {
        let (length, addr) = listener
            .recv_from(&mut hello_msg)
            .map_err(|error| format!("Failed to receive data from: {other_peer}: {error}"))?;

        if length == HELLO_MESSAGE.len()
            && &hello_msg[..length] == HELLO_MESSAGE
            && addr == other_peer
        {
            // established connection
            ping_sender.send(true).unwrap();
            break;
        }
    }
    ping_handle
        .join()
        .map_err(|error| format!("Could not finish thread: {error:?}"))
}

fn spawn_hello_ping(
    listener: Arc<UdpSocket>,
    socket_addr: SocketAddr,
) -> (Sender<bool>, JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();

    (
        sender,
        std::thread::spawn(move || {
            loop {
                if let Ok(msg) = receiver.try_recv()
                    && msg
                {
                    break;
                }
                listener.send_to(HELLO_MESSAGE, socket_addr).unwrap();
                std::thread::sleep(Duration::from_millis(100));
            }
        }),
    )
}
