use solana_client::nonblocking::pubsub_client::PubsubClient;
use std::{collections::VecDeque, sync::Arc};
use thiserror::Error;
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};

// Custom error types for our connection pool
#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Connection pool is empty")]
    PoolEmpty,
    #[error("Failed to create connection: {0}")]
    ConnectionError(String),
    #[error("Subscription error: {0}")]
    SubscriptionError(String),
}

// Structure representing a connection in our pool
struct PooledConnection {
    client: PubsubClient,
    // Track how many subscriptions are active on this connection
    subscription_count: usize,
}

impl PooledConnection {
    pub async fn get_client(self) -> PubsubClient {
        self.client
    }
}

// The connection pool itself
pub struct SolanaConnectionPool {
    ws_urls: Vec<String>,
    available_connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    max_subscriptions_per_connection: usize,
    max_connections: usize,
    active_connection_count: Arc<RwLock<usize>>,
}
