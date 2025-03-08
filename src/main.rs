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
    let rpc_client = Arc::new(RpcClient::new(
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
    let logs_filter = RpcTransactionLogsFilter::Mentions(vec![
        "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C".to_string(),
    ]);
    let websocket_sub_config = RpcTransactionLogsConfig {
        commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
    };

    // Set up subscription
    println!("subscription set");
    let wss = PubsubClient::new(&env::var("helius_websocket")?).await?;

    let (mut a, _b) = wss
        .logs_subscribe(logs_filter, websocket_sub_config)
        .await
        .unwrap();

    println!("Prosessing stream");
    while let Some(msg) = a.next().await {
        println!("------------------------------------------------------------");
        // Remove this log at one point in time: Optimization.
        log::info!("New transaction signature: {:?}", msg.value.signature);

        if let Some(tx_involved_pubkeys) = poolberry::common::decode_transaction(
            &rpc_client,
            msg.value.signature.parse::<Signature>().unwrap(),
        )
        .unwrap()
        {
            log::info!("Checking {:?} involved accounts!", tx_involved_pubkeys);
            // ******************************************************************************************
            check_if_token(
                &rpc_client,
                tx_involved_pubkeys,
                &not_token_hset,
                &mint_acc_bmap,
            )
            .unwrap();
        }
        log::info!("{:?}", mint_acc_bmap.sort_descending()?)
    }

    Ok(())
}
