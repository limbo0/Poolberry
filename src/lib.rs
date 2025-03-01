pub mod volumeprofile;

use anyhow::Result;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use std::sync::Arc;

type myclient = std::sync::Arc<RpcClient>;

// This will return all the accounts which was involved in the execution of the signature.
pub fn decode_transaction(rpc_client: &myclient, signature: Signature) -> Result<Vec<Pubkey>> {
    let txn_decode_config = RpcTransactionConfig {
        // Multiple options selectable; change depending on what you need.
        encoding: Some(solana_transaction_status_client_types::UiTransactionEncoding::Base58),
        commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };
    let decoded = rpc_client.get_transaction_with_config(&signature, txn_decode_config)?;
    Ok(decoded
        .transaction
        .transaction
        .decode()
        .unwrap()
        .message
        .static_account_keys()
        .to_vec())
}

// This should receive a list of new Pubkey and a list of predefined tokens which are not tokens
// filter and output only the ones which are tokens,
pub fn check_if_token(
    rpc_client: &myclient,
    accounts: Vec<Pubkey>,
    not_token: &Arc<NotToken>,
) -> Result<()> {
    let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<Pubkey>()?;
    let token_extension_program =
        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".parse::<Pubkey>()?;

    let acc: Vec<String> = accounts
        .into_iter()
        .filter_map(|pubk| {
            match rpc_client
                .get_token_account_with_commitment(&pubk, CommitmentConfig::processed())
                .ok()
            {
                Some(data) => Some(data.value.unwrap().mint),
                _ => None,
            }
        })
        .collect();

    log::info!("Filtered: {:#?}", acc);

    Ok(())
}

pub struct NotToken {
    data: Vec<Pubkey>,
}

impl NotToken {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}
