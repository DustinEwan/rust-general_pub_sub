use general_pub_sub::{Client, Message, PubSub};

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

    fn send(&mut self, message: &Message<&str>) {
        println!(
            "Client ({}) Received Message from Channel ({}): {}",
            self.id, message.source, message.contents
        );
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

    pubsub
        .sub_client(client_one, all_channels)
        .expect("This should not happen");

    pubsub.pub_message(channel_a, "Hello from Channel A");
    pubsub.pub_message(channel_b, "Hello from Channel B");
    pubsub.pub_message(channel_c, "Hello from Channel C");

    pubsub
        .sub_client(client_one, channel_a)
        .expect("This should not happen");

    pubsub.pub_message(channel_a, "Client 1 should only receive this once.");

    pubsub
        .unsub_client(client_one, all_channels)
        .expect("This should not happen");

    pubsub.pub_message(channel_b, "Nobody should receive this message");
}
