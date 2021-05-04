use general_pub_sub::{Client, PubSub};

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

    let channel_a = "channel.a";
    let channel_b = "channel.b";
    let channel_c = "channel.c";

    let all_channels = "channel.*";

    pubsub.add_client(client_one);

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_one, all_channels) {
        println!("This should not happen: {}", unexpected_error);
    }

    pubsub.pub_message(channel_a, "Hello from Channel A");
    pubsub.pub_message(channel_b, "Hello from Channel B");
    pubsub.pub_message(channel_c, "Hello from Channel C");

    if let Result::Err(unexpected_error) = pubsub.sub_client(client_one, channel_a) {
        println!("This should not happen: {}", unexpected_error);
    }

    pubsub.pub_message(channel_a, "Client 1 should only receive this once.");

    if let Result::Err(unexpected_error) = pubsub.unsub_client(client_one, all_channels) {
        println!("This should not happen: {}", unexpected_error);
    }

    pubsub.pub_message(channel_b, "Nobody should receive this message");
}
