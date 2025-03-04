use anyhow::Result;
use base64::Engine;
use dotenv::dotenv;
use log::LevelFilter;
use poolberry::{NotToken, check_if_token};
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::VersionedTransaction};
use solana_transaction_status_client_types::EncodedTransaction;
use std::{env, io::Write, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Global buffer blacklisted tokens.
    let not_token = Arc::new(NotToken::new());
    let rpc_client = Arc::new(RpcClient::new(
        "https://solana-mainnet.core.chainstack.com/8d701a8cf39221fedef455984ecd8b4f",
    ));
    // logger config and initialize
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

    // Config filters for data subscribing.
    let logs_filter = RpcTransactionLogsFilter::Mentions(vec![
        "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C".to_string(),
    ]);
    let websocket_sub_config = RpcTransactionLogsConfig {
        commitment: Some(solana_commitment_config::CommitmentConfig::processed()),
    };

    // Set up subscription
    println!("subscription set");
    let (a, b) = PubsubClient::logs_subscribe(
        &env::var("helius_websocket").expect("Failed to load helius_websocket from .env!"),
        logs_filter,
        websocket_sub_config,
    )?;

    println!("Prosessing stream");
    while let Some(msg) = b.recv().unwrap().into() {
        println!("------------------------------------------------------------");
        // Remove this log at one point in time: Optimization.
        log::info!("New transaction signature: {:?}", msg.value.signature);

        if let Some(accounts) =
            poolberry::decode_transaction(&rpc_client, msg.value.signature.parse::<Signature>()?)?
        {
            // This here should check if the address is a token account
            // If not then it should update the not_token list with a new value address which is not a
            // token.
            //
            check_if_token(&rpc_client, accounts, &not_token)?;
        }
        // log::info!("All involved accounts: {:#?}", accounts);

        // log::info!("NotToken: {:#?}", not_token);
    }

    Ok(())
}
