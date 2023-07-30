use mio::event::Event;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

const CONN_ACCEPT_TOKEN: Token = Token(0);

fn main() {
    println!("Starting Gredis server...");

    let mut poll = Poll::new().expect("Failed to create poll instance");
    let mut events = Events::with_capacity(128);

    let address = "127.0.0.1:6379".parse().expect("Invalid address");
    let mut listener = TcpListener::bind(address).expect("Failed to bind TCP listener");

    poll.registry()
        .register(&mut listener, CONN_ACCEPT_TOKEN, Interest::READABLE)
        .expect("Failed to register listener");

    loop {
        poll.poll(&mut events, None).expect("Failed to poll events");

        for event in events.iter() {
            match event.token() {
                CONN_ACCEPT_TOKEN => {
                    let (mut stream, _) = listener.accept().expect("Failed to accept connection");
                    println!("accepted new connection");
                    let mut buf = [0; 512];
                    poll.registry()
                        .register(&mut stream, STREAM_READ_TOKEN, Interest::READABLE)
                        .expect("Failed to register TCP stream");
                }
                STREAM_READ_TOKEN => {
                    let mut buffer = vec![0u8; 1024];
                    match stream.read(&mut buffer).await {
                        Ok(n) if n > 0 => {
                            let data = &buffer[..n];
                            let received_text = String::from_utf8_lossy(data);
                            println!("Received: {}", received_text);
                        }
                        Ok(_) => {
                            // The stream was closed by the remote end
                            println!("Connection closed by remote end");
                            // Remove the stream from the event loop
                            poll.registry()
                                .deregister(&mut stream)
                                .expect("Failed to deregister TCP stream");
                        }
                        Err(e) => {
                            eprintln!("Error while reading from stream: {}", e);
                            // Remove the stream from the event loop
                            poll.registry()
                                .deregister(&mut stream)
                                .expect("Failed to deregister TCP stream");
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
