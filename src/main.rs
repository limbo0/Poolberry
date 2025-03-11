use anyhow::Result;
use dotenv::dotenv;
use futures_util::{StreamExt, stream::Next};
use log::LevelFilter;
use poolberry::common::{MintAccounts, NotToken, SolanaConnectionPool, check_if_token};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::VersionedTransaction};
use solana_transaction_status_client_types::EncodedTransaction;
use std::{env, io::Write, sync::Arc, time::Duration};
use tokio::{task::JoinSet, time};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Global buffer blacklisted tokens.
    let not_token_hset = Arc::new(NotToken::new());
    let mint_acc_bmap = Arc::new(MintAccounts::new());
    let http_client = Arc::new(RpcClient::new(
        env::var("chainstack_https").expect("chainstack_https env not found!"),
    ));
    // ************************************************************************************************************
    let mut builder = env_logger::Builder::new();
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: {}: {}",
                buf.timestamp(),
                record.level(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .init();
    // ************************************************************************************************************
    // Pooling
    let mut wss_urls = Vec::with_capacity(5);
    wss_urls.push(env::var("helius_websocket").expect("No env var found!"));
    wss_urls.push(env::var("chainstack_websocket").expect("No env var found!"));

    // ************************************************************************************************************
    // Config filters for data subscribing.

    let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

    let mut set = JoinSet::new();
    let http_client1 = Arc::clone(&http_client);
    let sender = Arc::new(sender);
    let sender1 = Arc::clone(&sender);

    set.spawn(async move {
        // Set up subscription

        let wss = PubsubClient::new(&env::var("helius_websocket").unwrap())
            .await
            .unwrap();
        println!("subscribing!");
        let (mut a, _b) = wss
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![String::from(
                    "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
                )]),
                RpcTransactionLogsConfig {
                    commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
                },
            )
            .await
            .unwrap();

        while let Some(msg) = a.next().await {
            println!("------------------------------------------------------------");
            // Remove this log at one point in time: Optimization.
            log::info!("CPMM new transaction signature: {:?}", msg.value.signature);

            // The unwrap of this data is necessary to avoid sending None data.
            // This will ensure the channel will always send a valid wrapped data.
            if let Some(data) = poolberry::common::decode_transaction(
                &http_client1,
                msg.value.signature.parse::<Signature>().unwrap(),
            )
            .unwrap()
            {
                log::info!("Sending data form task: {:?}", tokio::task::id());
                sender1
                    .send(Some(data))
                    .await
                    .expect("Failed to send tx_involved_pubkeys via channel!");
            }
        }
    });

    let http_client2 = Arc::clone(&http_client);
    set.spawn(async move {
        let wss = PubsubClient::new(&env::var("helius_websocket").unwrap())
            .await
            .unwrap();

        let (mut a, _b) = wss
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![
                    (String::from("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK")),
                ]),
                RpcTransactionLogsConfig {
                    commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
                },
            )
            .await
            .unwrap();

        while let Some(msg) = a.next().await {
            println!("------------------------------------------------------------");
            // Remove this log at one point in time: Optimization.
            log::info!("CAMM new transaction signature: {:?}", msg.value.signature);

            // The unwrap of this data is necessary to avoid sending None data.
            // This will ensure the channel will always send a valid wrapped data.
            if let Some(data) = poolberry::common::decode_transaction(
                &http_client2,
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
    });

    let http_client3 = Arc::clone(&http_client);
    set.spawn(async move {
        while let Some(data) = receiver.recv().await.unwrap() {
            log::info!("Checking {:?} involved accounts!", data);
            check_if_token(&http_client3, data, &not_token_hset, &mint_acc_bmap)
                .await
                .unwrap();
            log::info!("{:?}", mint_acc_bmap.sort_descending().unwrap())
        }
    });

    set.join_all().await;
    Ok(())
}
