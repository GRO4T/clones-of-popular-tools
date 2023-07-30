extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
extern crate mio;

use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Shutdown, SocketAddr};

use dotenv::dotenv;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};

struct Client {
    stream: mio::net::TcpStream,
}

const CONN_ACCEPT_TOKEN: Token = Token(0);

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let listen_ip_addr: IpAddr = dotenv!["LISTEN_IP_ADDR"].parse()?;
    let listen_ip_port: u16 = dotenv!["LISTEN_IP_PORT"].parse()?;
    let max_clients: usize = dotenv!["MAX_CLIENTS"].parse()?;
    let max_message_bytes: usize = dotenv!["MAX_MESSAGE_BYTES"].parse()?;

    let mut poll = Poll::new().expect("Failed to create poll instance");
    let mut events = Events::with_capacity(128);
    let mut next_client_id: usize = 1;
    // Better way is probably to use LruCache from https://docs.rs/lru/latest/lru/,
    // but I wanted to learn the standard collection.
    let mut connections: HashMap<usize, Client> = HashMap::with_capacity(max_clients);
    let mut buffer: Vec<u8> = Vec::with_capacity(max_message_bytes);
    for _ in 0..max_message_bytes {
        buffer.push(0);
    } // Set len = capacity

    let listen_addr = SocketAddr::new(listen_ip_addr, listen_ip_port);
    let mut listener = TcpListener::bind(listen_addr).expect(
        &format!(
            "Failed to bind TCP listener on {}:{}",
            listen_ip_addr, listen_ip_port
        )
        .to_string(),
    );

    println!(
        "Starting Gredis server on {}:{}...",
        listen_ip_addr, listen_ip_port
    );

    poll.registry()
        .register(&mut listener, CONN_ACCEPT_TOKEN, Interest::READABLE)
        .expect("Failed to register listener");

    loop {
        poll.poll(&mut events, None).expect("Failed to poll events");

        for event in events.iter() {
            match event.token() {
                CONN_ACCEPT_TOKEN => {
                    // From mio: ALWAYS operate within a loop, and read until WouldBlock
                    loop {
                        match listener.accept() {
                            Ok((mut stream, _addr)) => {
                                poll.registry()
                                    .register(
                                        &mut stream,
                                        Token(next_client_id),
                                        Interest::READABLE | Interest::WRITABLE,
                                    )
                                    .expect("Failed to register client read/write socket");
                                connections.insert(next_client_id, Client { stream });
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(e) => panic!("Unexpected error={:?}", e),
                        }
                    }
                }
                Token(token_id) => {
                    if event.is_readable() {
                        let mut closed = false;
                        if let Some(_client) = connections.get_mut(&token_id) {
                            println!("Received a message!");
                            // if let Ok(req) = self.receive(&mut client.stream, &mut buffer) {
                            //     println!("Received a message!");
                            // } else {
                            //     poll.registry().deregister(&mut client.stream)?;
                            //     client.stream.shutdown(Shutdown::Both)?;
                            //     closed = true;
                            // }
                        } else {
                            println!("ERROR: unknown token id={}", token_id);
                        }
                        if closed {
                            // FIXME: salvage `token_id` value
                            connections.remove(&token_id);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
