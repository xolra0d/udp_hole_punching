use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::LazyLock;
use tokio::net::UdpSocket;

const IDENTIFIER_LENGTH: usize = 32; // UUID is 32 bytes length
static CONNECTION_INFO: LazyLock<DashMap<[u8; IDENTIFIER_LENGTH], SocketAddr>> =
    LazyLock::new(DashMap::new);

#[tokio::main]
async fn main() -> Result<(), String> {
    let bind_addr = std::env::args().nth(1).ok_or(
        "Server bind address missing. Expected: `./server BIND_ADDRESS:BIND_PORT".to_string(),
    )?;

    let listener = UdpSocket::bind(&bind_addr)
        .await
        .map_err(|error| format!("Failed to bind to {bind_addr}: {error}.",))?;

    let mut client_unique_identifier = [0u8; IDENTIFIER_LENGTH];
    loop {
        let (len, first_peer) = match listener.recv_from(&mut client_unique_identifier).await {
            Ok(result) => result,
            Err(error) => {
                eprintln!("ERROR: Failed to receive bytes: {error}.",);
                continue;
            }
        };
        if len != IDENTIFIER_LENGTH {
            continue;
        }

        if let Some((_, waiting_peer)) = CONNECTION_INFO.remove(&client_unique_identifier) {
            if let Err(error) = announce_peer(&listener, first_peer, waiting_peer).await {
                eprintln!("ERROR: {error}",);
                continue;
            }
            if let Err(error) = announce_peer(&listener, waiting_peer, first_peer).await {
                eprintln!("ERROR: {error}",);
            }
        } else {
            CONNECTION_INFO.insert(client_unique_identifier, first_peer);
        }
    }
}

async fn announce_peer(
    listener: &UdpSocket,
    peer: SocketAddr,
    dst_peer: SocketAddr,
) -> Result<(), String> {
    let bytes = peer.to_string();
    let bytes = bytes.as_bytes();
    let bytes = [[bytes.len() as u8].as_slice(), bytes].concat();

    let len = listener
        .send_to(&bytes, dst_peer)
        .await
        .map_err(|error| format!("Failed to send peer info ({peer} to {dst_peer}): {error}."))?;

    if len != bytes.len() {
        return Err(format!(
            "Expected to send {} bytes, but only sent {len} bytes to address {dst_peer}.",
            bytes.len()
        ));
    }

    Ok(())
}
