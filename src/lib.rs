use std::{collections::{ BTreeSet, HashMap }, hash::Hash};
use std::error::Error;
use std::marker::PhantomData;

/// A Unique Identifier
/// 
/// The "unique" aspect of this trait is enforced within the PubSub
/// itself.  However, in addition to being unique, the identifier must
/// implement (or derive) core::cmp::Ord and std::hash::Hash. 
pub trait UniqueIdentifier: Ord + Hash {}
impl<TIdentifier: Ord + Hash> UniqueIdentifier for TIdentifier {}

/// A PubSub Client
///
/// Trait describing a generic PubSub Client.
///
/// The identifier can be any data type so long as it conforms to
/// the `UniqueIdentifier` trait.
///
/// Message can also be of any type.
///
/// # Examples
/// 
/// Basic Usage:
///
/// ```
/// struct BasicClient {
///   id: u32   
/// }
///
/// impl Client<u32, String> for BasicClient {
///   fn get_id(&self) -> u32 {
///      return self.id;
///   }
///
///   fn send(&self, message: String) {
///       println!("Client ({}) Received: {}", self.id, message); 
///   }
/// }
/// ```
/// 
/// Multi-client Example:
/// 
/// ```
/// struct ConsoleClient {
///   id: u32
/// }
/// 
/// impl Client<u32, String> for ConsoleClient {
///   fn get_id(&self) -> u32 {
///      return self.id;
///   }
///
///   fn send(&self, message: String) {
///       println!("Client ({}) Received: {}", self.id, message); 
///   }
/// }
/// 
/// struct TcpClient {
///   id: String,
///   stream: std::net::TcpStream
/// }
/// 
/// impl Client<String, String> for TcpClient {
///   fn get_id(&self) -> String {
///     return self.id;
///   }
/// 
///   fn send(&self, message: String) {
///     self.stream.write(format!("Client ({}) Received: {}", self.id, message).as_bytes())
///   }
/// }
/// 
/// enum Clients {
///   Console(ConsoleClient),
///   Tcp(TcpClient)
/// }
/// 
/// impl Client<String, String> for Clients {
///   fn get_id(&self) -> String {
///     match self {
///       Self::Console(client) => client.get_id().to_string(),
///       Self::Tcp(client) => client.get_id()
///     }
///   }
/// 
///   fn send(&self, message: String) {
///     match self {
///       Self::Console(client) => client.send(message),
///       Self::Console(client) => client.send(message)
///     }
///   }
/// }
/// ```
pub trait Client<TIdentifier: UniqueIdentifier, TMessage> {
    /// Gets the `ID` of the `Client`. Must be unique.
    fn get_id(&self) -> TIdentifier;

    /// Sends a `Message` to a `Client`.
    fn send(&self, message: &TMessage);
}

/// PubSubError is used for errors specific to `PubSub` (such as adding or removing `Client`s)
#[derive(Debug)]
pub struct PubSubError { details: String }

impl Error for PubSubError {}
impl std::fmt::Display for PubSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.details);
    }
}

/// A PubSub
pub struct PubSub<TClient: Client<TIdentifier, TMessage>, TIdentifier: UniqueIdentifier, TMessage> {
    clients: HashMap<TIdentifier, TClient>,
    channels: HashMap<String, BTreeSet<TIdentifier>>,
    phantom: PhantomData<TMessage>
}


/// Implementation for a `PubSub`
/// 
/// The standard workflow for a `PubSub` is to:
/// 
/// 1. Create a new `PubSub`.
/// 2. Add one or more `Clients`.
/// 3. Subscribe the `Clients` to `Channels` of interest.
/// 4. Publish `Messages` to the `Channels`. The `Message` is broadcast to all `Clients` subscribed to the `Channel`.
impl<TClient: Client<TIdentifier, TMessage>, TIdentifier: UniqueIdentifier, TMessage: Clone> PubSub<TClient, TIdentifier, TMessage> {
    
    /// Creates a new `PubSub`
    /// 
    /// All `Clients` of the `PubSub` must use the same type of `Identifier`
    /// and receive the same type of `Message`.
    pub fn new() -> PubSub<TClient, TIdentifier, TMessage> {
        PubSub {
            clients: HashMap::new(),
            channels: HashMap::new(),
            phantom: PhantomData
        }
    }

    /// Adds a `Client` to the `PubSub`
    pub fn add_client(&mut self, client: TClient) {
        let token = client.get_id();
        self.clients.insert(token, client);
    }

    // Unsubscribes a `Client` from all `Channels` and removes the `Client` from the `PubSub`.
    pub fn remove_client(&mut self, client: TClient) {
        let token = &client.get_id();
        self.clients.remove(token);

        for subbed_clients in self.channels.values_mut() {
            subbed_clients.remove(token);
        }
    }

    /// Subscribes a `Client` to a `Channel`.
    ///
    /// Results in a `PubSubError` when a `Client` attempts to subscribe to a 
    /// `Channel` that it is already subscribed to.
    pub fn sub_client(&mut self, client: TClient, channel: String) -> Result<(), PubSubError> {
        let subbed_clients = self.channels.entry(channel).or_insert(BTreeSet::new());

        let result = subbed_clients.insert(client.get_id());

        return if result {
            Ok(())
        } else {
            Err(PubSubError { details: "Client already subscribed to this channel.".to_owned() })
        }
    }

    /// Unsubscribes a `Client` from a `Channel`
    /// 
    /// Results in a `PubSubError` when a `Client` attempts to unsubscribe
    /// from a `Channel` it is not subscribed to.
    pub fn unsub_client(&mut self, client: TClient, channel: String) -> Result<(), PubSubError> {
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            subbed_clients.remove(&client.get_id());
            return Ok(())
        } else {
            Err(PubSubError { details: "Client was not subscribed to this channel.".to_owned() })
        }
    }

    /// Publishes a `Message` to all `Clients` subscribed to the provided `Channel`.
    pub fn pub_message(&mut self, channel: String, msg: &TMessage) {
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            for token in subbed_clients.iter() {
                if let Some(client) = self.clients.get(token) {
                    client.send(msg);
                }
            }
        }
    }
}