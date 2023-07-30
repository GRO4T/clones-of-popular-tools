extern crate mio;

use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr};

struct Client {
    stream: mio::net::TcpStream,
}

const CONN_ACCEPT_TOKEN: Token = Token(0);

fn main() -> Result<(), Box<dyn Error>> {
    let listen_ip_addr: IpAddr = env::var("LISTEN_IP_ADDR")?.parse()?;
    let listen_ip_port: u16 = env::var("LISTEN_IP_PORT")?.parse()?;
    let max_clients: usize = env::var("MAX_CLIENTS")?.parse()?;
    let max_message_bytes: usize = env::var("MAX_MESSAGE_BYTES")?.parse()?;

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
                                next_client_id += 1;
                                println!("Accepted a new connection");
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(e) => panic!("Unexpected error={:?}", e),
                        }
                    }
                }
                Token(client_id) => {
                    if event.is_readable() {
                        let mut closed = false;
                        if let Some(_client) = connections.get_mut(&client_id) {
                            let mut offset = 0;
                            loop {
                                match _client.stream.read(&mut buffer[offset..]) {
                                    Ok(0) => {
                                        if offset > 0 {
                                            break;
                                        } else {
                                            panic!("Connection aborted");
                                        }
                                    }
                                    Ok(n_bytes) => {
                                        offset += n_bytes;
                                        if offset == max_message_bytes {
                                            println!("ERROR: increase buffer capacity!");
                                            panic!("Buffer too small");
                                        }
                                    }
                                    Err(ref e)
                                        if e.kind() == std::io::ErrorKind::WouldBlock
                                            || e.kind() == std::io::ErrorKind::Interrupted =>
                                    {
                                        break
                                    }
                                    Err(e) => panic!("Unexpected error={:?}", e),
                                }
                            }

                            println!(
                                "Received from client {}: {}",
                                client_id,
                                std::str::from_utf8(&buffer).unwrap()
                            );

                            if offset > 0 {
                                for i in 0..offset {
                                    buffer[i] = 0; // Ensure no payload leakage
                                }
                            } else {
                                panic!("Empty message");
                            }

                            _client.stream.write(b"+PONG\r\n")?;
                        } else {
                            println!("ERROR: unknown token id={}", client_id);
                        }
                        if closed {
                            // FIXME: salvage `token_id` value
                            connections.remove(&client_id);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
