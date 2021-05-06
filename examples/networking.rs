use general_pub_sub::{Client, Message, PubSub};
use std::{
    borrow::Borrow,
    io::BufRead,
    net::{TcpListener, TcpStream},
};
use std::{
    io::{BufReader, Write},
    net::SocketAddr,
};

struct TcpClient {
    id: SocketAddr,
    stream: TcpStream,
}

impl Clone for TcpClient {
    fn clone(&self) -> TcpClient {
        TcpClient {
            id: self.id,
            stream: self.stream.try_clone().expect("Failed to clone TCP Stream"),
        }
    }
}

impl TcpClient {
    pub fn new(id: SocketAddr, stream: TcpStream) -> TcpClient {
        TcpClient { id, stream }
    }
}

impl Client<SocketAddr, &str> for TcpClient {
    fn get_id(&self) -> SocketAddr {
        self.id
    }

    fn send(&mut self, message: &Message<&str>) {
        if let Result::Err(error) = self.stream.write(
            format!(
                "Client ({}) Received Message from Channel ({}): {}\n",
                self.id, message.source, message.contents
            )
            .as_bytes(),
        ) {
            println!("Failed to write response to client: {}", error);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");

    let channel = "clients.all";

    let mut pubsub = PubSub::new();

    for _ in 0..5 {
        std::thread::spawn(move || match TcpStream::connect("localhost:3333") {
            Ok(stream) => {
                println!("Successfully connected to server. Awaiting messages from channel.");

                let reader = BufReader::new(stream);
                for message in reader.lines() {
                    println!(
                        "Received message from server:\n\t{}",
                        message.expect("Could not read message.")
                    );
                }
            }
            Err(e) => {
                println!("Failed to connect to server: {}", e);
            }
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let ip_addr = stream.peer_addr().unwrap();
                println!("New connection: {}", ip_addr);
                let client = TcpClient::new(ip_addr, stream);
                pubsub.add_client(client.clone());

                pubsub
                    .sub_client(client.clone(), channel)
                    .expect("Failed to subscribe to channel.");

                // THIS IS NAUGHTY!  DON'T DO THIS IN REAL LIFE!
                let message = &*Box::leak(
                    format!("A new client ({}) pubsub server!", ip_addr).into_boxed_str(),
                );

                pubsub.pub_message(channel, message);
            }
            Err(e) => {
                println!("Error establishing connection: {}", e);
            }
        }
    }
}
