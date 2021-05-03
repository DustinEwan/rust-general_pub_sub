use general_pub_sub::{Client, PubSub, PubSubError};

#[derive(Clone, Copy)]
struct BasicClient {
    id: u32
}

impl BasicClient {
    pub fn new(id: u32) -> BasicClient {
        BasicClient {
            id
        }
    }
}

impl Client<u32, &str> for BasicClient {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn send(&self, message: &str) {
        println!("Client ({}) Received: {}", self.id, message);
    }
}


fn main() {
    let mut pubsub = PubSub::new();

    let client_one = BasicClient::new(1);
    let client_two = BasicClient::new(2);
    let client_three = BasicClient::new(3);

    let channel_a = "Channel A";
    let channel_b = "Channel B";
    let channel_c = "Channel C";

    let all_channels = "Channel *";

    pubsub.add_client(client_one);
    pubsub.add_client(client_two);
    pubsub.add_client(client_three);

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_one, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_two, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_one, channel_b) {
        println!("This should not happen: {}", unexpected_error);
    }

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_three, all_channels) {
        println!("This should not happen: {}", unexpected_error);
    }

    println!("");
    println!("Sending a message to Channel A");
    pubsub.pub_message(channel_a, "All clients should receive this message.");

    println!("");
    println!("Sending a message to Channel B");
    pubsub.pub_message(channel_b, "Clients 1 and 3 should receive this message.");

    if let Result::Err(unexpected_error) = pubsub.unsub_client(client_one, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    println!("");
    println!("Sending a message to Channel A");
    pubsub.pub_message(channel_a, "Clients 2 and 3 should receive this message.");

    pubsub.remove_client(client_one);
    pubsub.remove_client(client_two);

    println!("");
    println!("Sending a message to Channel A");
    pubsub.pub_message(channel_a, "Client 3 should receive this message.");
    
    println!("");
    println!("Sending a message to Channel C");
    pubsub.pub_message(channel_c, "Nobody ever subscribed to this channel, but Client 3 should receive it.");

    println!("");
    println!("Attempting to unsubscribe Client One after being removed.");
    if let Result::Err(expected_error) = pubsub.unsub_client(client_one, channel_a) {
        match expected_error {
            PubSubError::ClientNotSubscribedError => println!("This error is expected: {}", expected_error),
            _ => println!("This should not happen: {}", expected_error)
        }
    }
}