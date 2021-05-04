use general_pub_sub::{Client, PubSub, PubSubError};

#[derive(Clone, Copy)]
struct BasicClient {
    id: u32,
}

impl BasicClient {
    pub fn new(id: u32) -> BasicClient {
        BasicClient { id }
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

    let channel_a = "channel.a";
    let channel_b = "channel.b";

    pubsub.add_client(client_one);
    pubsub.add_client(client_two);

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_one, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_two, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_one, channel_b) {
        println!("This should not happen: {}", unexpected_error);
    }

    pubsub.pub_message(channel_a, "Both clients should receive this message.");
    pubsub.pub_message(channel_b, "Only Client 1 should receive this message.");

    if let Result::Err(unexpected_error) = pubsub.unsub_client(client_one, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    pubsub.pub_message(channel_a, "Only Client 2 should receive this message.");

    pubsub.remove_client(client_one);

    pubsub.pub_message(channel_a, "Nobody should receive this message.");

    if let Result::Err(expected_error) = pubsub.unsub_client(client_one, channel_a) {
        match expected_error {
            PubSubError::ClientNotSubscribedError => {
                println!("This error is expected: {}", expected_error)
            }
            _ => println!("This should not happen: {}", expected_error),
        }
    }
}
