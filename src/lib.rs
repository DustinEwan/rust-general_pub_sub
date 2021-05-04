use itertools::Itertools;
use std::error::Error;
use std::marker::PhantomData;
use std::{
    collections::{BTreeSet, HashMap},
    hash::Hash,
};
use wildmatch::WildMatch;

/// A Unique Identifier
///
/// The "unique" aspect of this trait is enforced within the PubSub
/// itself.  However, in addition to being unique, the identifier must
/// implement (or derive) core::cmp::Ord and std::hash::Hash.
pub trait UniqueIdentifier: Ord + Eq + Hash {}
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
/// impl Client<u32, &str> for BasicClient {
///   fn get_id(&self) -> u32 {
///      return self.id;
///   }
///
///   fn send(&self, message: &str) {
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
/// impl Client<u32, &str> for ConsoleClient {
///   fn get_id(&self) -> u32 {
///      return self.id;
///   }
///
///   fn send(&self, message: &str) {
///       println!("Client ({}) Received: {}", self.id, message);
///   }
/// }
///
/// struct TcpClient {
///   id: &str,
///   stream: std::net::TcpStream
/// }
///
/// impl Client<&str, &str> for TcpClient {
///   fn get_id(&self) -> &str {
///     return self.id;
///   }
///
///   fn send(&self, message: &str) {
///     self.stream.write(format!("Client ({}) Received: {}", self.id, message).as_bytes())
///   }
/// }
///
/// enum Clients {
///   Console(ConsoleClient),
///   Tcp(TcpClient)
/// }
///
/// impl Client<&str, &str> for Clients {
///   fn get_id(&self) -> &str {
///     match self {
///       Self::Console(client) => client.get_id().to_string(),
///       Self::Tcp(client) => client.get_id()
///     }
///   }
///
///   fn send(&self, message: &str) {
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
    fn send(&self, message: TMessage);
}

/// PubSubError is used for errors specific to `PubSub` (such as adding or removing `Client`s)
#[derive(Debug)]
pub enum PubSubError {
    ClientAlreadySubscribedError,
    ClientNotSubscribedError,
    ChannelDoesNotExistError,
    ClientWithIdentifierAlreadyExistsError,
    ClientDoesNotExistError,
}

impl Error for PubSubError {}
impl std::fmt::Display for PubSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientAlreadySubscribedError => {
                write!(f, "Client already subscribed to channel.")
            }
            Self::ClientNotSubscribedError => write!(f, "Client is not subscribed to channel."),
            Self::ChannelDoesNotExistError => write!(f, "Channel does not exist."),
            Self::ClientDoesNotExistError => write!(f, "Client does not exist."),
            Self::ClientWithIdentifierAlreadyExistsError => {
                write!(f, "Client with that identifier already exists.")
            }
        }
    }
}

/// A PubSub
pub struct PubSub<
    'a,
    TClient: Client<TIdentifier, TMessage>,
    TIdentifier: UniqueIdentifier,
    TMessage,
> {
    clients: HashMap<TIdentifier, TClient>,
    channels: HashMap<&'a str, BTreeSet<TIdentifier>>,
    pattern_channels: HashMap<&'a str, BTreeSet<TIdentifier>>,
    phantom: PhantomData<TMessage>,
}

fn channel_is_pattern(channel: &str) -> bool {
    channel.contains('*') || channel.contains('?')
}

/// Implementation for a `PubSub`
///
/// The standard workflow for a `PubSub` is to:
///
/// 1. Create a new `PubSub`.
/// 2. Add one or more `Clients`.
/// 3. Subscribe the `Clients` to `Channels` of interest.
/// 4. Publish `Messages` to the `Channels`. The `Message` is broadcast to all `Clients` subscribed to the `Channel`.
impl<
        'a,
        TClient: Client<TIdentifier, TMessage>,
        TIdentifier: UniqueIdentifier,
        TMessage: Clone + Copy,
    > PubSub<'a, TClient, TIdentifier, TMessage>
{
    /// Creates a new `PubSub`
    ///
    /// All `Clients` of the `PubSub` must use the same type of `Identifier`
    /// and receive the same type of `Message`.
    pub fn new() -> PubSub<'a, TClient, TIdentifier, TMessage> {
        PubSub {
            clients: HashMap::new(),
            channels: HashMap::new(),
            pattern_channels: HashMap::new(),
            phantom: PhantomData,
        }
    }

    /// Adds a `Client` to the `PubSub`
    pub fn add_client(&mut self, client: TClient) {
        let token = client.get_id();
        self.clients.insert(token, client);
    }

    // Unsubscribes a `Client` from all `Channels` and removes the `Client` from the `PubSub`.
    pub fn remove_client(&mut self, client: TClient) {
        let identifier = &client.get_id();
        self.clients.remove(identifier);

        for subbed_clients in self.channels.values_mut() {
            subbed_clients.remove(identifier);
        }

        for subbed_clients in self.pattern_channels.values_mut() {
            subbed_clients.remove(identifier);
        }
    }

    fn get_channels_for_subscription(
        &mut self,
        channel: &'a str,
    ) -> &mut HashMap<&'a str, BTreeSet<TIdentifier>> {
        match channel_is_pattern(channel) {
            true => &mut self.pattern_channels,
            false => &mut self.channels,
        }
    }

    /// Subscribes a `Client` to a `Channel`.
    ///
    /// Results in a `PubSubError` when a `Client` attempts to subscribe to a
    /// `Channel` that it is already subscribed to.
    pub fn sub_client(&mut self, client: TClient, channel: &'a str) -> Result<(), PubSubError> {
        let target_channels = self.get_channels_for_subscription(channel);

        let subbed_clients = target_channels.entry(channel).or_insert_with(BTreeSet::new);

        let result = subbed_clients.insert(client.get_id());

        if result {
            Ok(())
        } else {
            Err(PubSubError::ClientAlreadySubscribedError)
        }
    }

    /// Unsubscribes a `Client` from a `Channel`
    ///
    /// Results in a `PubSubError` when a `Client` attempts to unsubscribe
    /// from a `Channel` it is not subscribed to.
    pub fn unsub_client(&mut self, client: TClient, channel: &'a str) -> Result<(), PubSubError> {
        let target_channels = self.get_channels_for_subscription(channel);

        if let Some(subbed_clients) = target_channels.get_mut(channel) {
            match subbed_clients.remove(&client.get_id()) {
                true => Ok(()),
                false => Err(PubSubError::ClientNotSubscribedError),
            }
        } else {
            Err(PubSubError::ChannelDoesNotExistError)
        }
    }

    /// Publishes a `Message` to all `Clients` subscribed to the provided `Channel`.
    pub fn pub_message<TInputMessage: Into<TMessage>>(
        &mut self,
        channel: &str,
        msg: TInputMessage,
    ) {
        let msg_ref = msg.into();

        let pattern_client_identifiers = self
            .pattern_channels
            .iter()
            .filter(|(pattern, _)| WildMatch::new(pattern) == channel)
            .map(|(_, clients)| clients.iter())
            .flatten();

        let subbed_clients = self.channels.get_mut(channel);
        let subbed_client_identifiers = subbed_clients.iter().map(|client| client.iter()).flatten();

        let unique_client_identifiers = subbed_client_identifiers
            .chain(pattern_client_identifiers)
            .unique();

        for identifier in unique_client_identifiers {
            if let Some(client) = self.clients.get(identifier) {
                client.send(msg_ref);
            }
        }
    }
}

impl<
        'a,
        TClient: Client<TIdentifier, TMessage>,
        TIdentifier: UniqueIdentifier,
        TMessage: Clone + Copy,
    > Default for PubSub<'a, TClient, TIdentifier, TMessage>
{
    fn default() -> Self {
        Self::new()
    }
}
