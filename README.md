# udp_hole_punching

Server (`server.rs`) helps to initiate [UDP Hole Punching](https://en.wikipedia.org/wiki/UDP_hole_punching) session using unique identifiers.

Algorithm:
1. Peer1 sends 32 byte unique self-generated identifier (e.g., UUID). Server records this identifier to lookup table with Peer1 socket address (IP + Port).
2. Peer2 sends the same identifier. Server finds out, that this identifier is already in lookup table, thus sends to Peer1 socket address of Peer2, and sends to Peer2 socket address of Peer1. Removes identifier entry.

For pure demonstration, example client (`client.rs`) is included.

Example session:
1. Server starts: `cargo run --bin server 127.0.0.1:33333`.
2. Peer1 starts: `cargo run --bin client 127.0.0.1:33333 127.0.0.1:33334 25ae167975a347c1898742cab2c830ad`
3. Peer2 starts: `cargo run --bin client 127.0.0.1:33333 127.0.0.1:33335 25ae167975a347c1898742cab2c830ad`
4. Peer1 receives: `Received peer address: 127.0.0.1:33335`
5. Peer2 receives: `Received peer address: 127.0.0.1:33334`

Formats:
* Server startup format: `./server BIND_ADDRESS:BIND_PORT`
* Client startup format: `./client SERVER_ADDRESS:SERVER_PORT CLIENT_ADDRESS:CLIENT_PORT 32BYTE_TOKEN`
* Server response with peer socket address: `<1 BYTE LENGTH OF SOCKET ADDR SENT><SOCKET ADDR IN UTF8>`
