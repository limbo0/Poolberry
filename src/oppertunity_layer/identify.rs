//! Continuously monitor markets and detect price discrepancies across trading venues with minimal latency.

use futures_util::StreamExt;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{env, sync::Arc};
use tokio::sync::mpsc::Sender;

// Logs subscribe magic sauce with filters.
pub async fn magic(
    http_client: Arc<RpcClient>,
    contract_address: String,
    sender: Arc<Sender<Option<Vec<Pubkey>>>>,
) {
    let wss = PubsubClient::new(&env::var("helius_websocket").unwrap())
        .await
        .unwrap();

    let (mut a, _b) = wss
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![contract_address]),
            RpcTransactionLogsConfig {
                commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
            },
        )
        .await
        .unwrap();

    while let Some(msg) = a.next().await {
        println!("------------------------------------------------------------");
        log::info!("New transaction signature: {:?}", msg.value.signature);

        // The unwrap of this data is necessary to avoid sending None data.
        // This will ensure the channel will always send a valid wrapped data.
        if let Some(data) = crate::common::decode_transaction(
            &http_client,
            msg.value.signature.parse::<Signature>().unwrap(),
        )
        .unwrap()
        {
            log::info!("Sending data form task: {:?}", tokio::task::id());
            sender
                .send(Some(data))
                .await
                .expect("Failed to send tx_involved_pubkeys via channel!");
        }
    }
}
