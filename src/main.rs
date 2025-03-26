use anyhow::Result;
use dotenv::dotenv;
use log::LevelFilter;
use poolberry::common::{MintAccounts, NotToken, check_if_token};
use solana_client::rpc_client::RpcClient;
use std::{env, io::Write, sync::Arc};
use tokio::task::JoinSet;

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
    let sender = Arc::new(sender);

    // ************************************************************************************************************
    let http_client1 = Arc::clone(&http_client);
    let sender1 = Arc::clone(&sender);
    set.spawn(async move {
        poolberry::oppertunity_layer::identify::magic(
            http_client1,
            String::from("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"),
            sender1,
        )
        .await;
    });

    // ************************************************************************************************************
    let http_client2 = Arc::clone(&http_client);
    set.spawn(async move {
        poolberry::oppertunity_layer::identify::magic(
            http_client2,
            String::from("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK"),
            Arc::clone(&sender),
        )
        .await;
    });

    // ************************************************************************************************************
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
