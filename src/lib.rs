pub mod volumeprofile;

use anyhow::Result;
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{account::Account, blake3::Hash, pubkey::Pubkey, signature::Signature};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

type Myclient = std::sync::Arc<RpcClient>;
// let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<Pubkey>()?;
// let token_extension_program =
//     "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".parse::<Pubkey>()?;

// This will return all the accounts which was involved in the execution of the signature.
pub fn decode_transaction(
    rpc_client: &Myclient,
    signature: Signature,
) -> Result<Option<Vec<Pubkey>>> {
    let txn_decode_config = RpcTransactionConfig {
        // Multiple options selectable; change depending on what you need.
        encoding: Some(solana_transaction_status_client_types::UiTransactionEncoding::Base58),
        commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };
    match rpc_client
        .get_transaction_with_config(&signature, txn_decode_config)
        .ok()
    {
        Some(value) => match value.transaction.meta.unwrap().err {
            Some(_err) => Ok(None),
            None => Ok(Some(
                value
                    .transaction
                    .transaction
                    .decode()
                    .expect("Failed to decode transaction!")
                    .message
                    .static_account_keys()
                    .to_vec(),
            )),
        },
        None => Ok(None),
    }
}

// This should receive a list of new Pubkey and a list of predefined tokens which are not tokens
// Add pkeys which are not tokens to the not token list.
pub fn check_if_token(
    rpc_client: &Myclient,
    accounts: Vec<Pubkey>,
    not_token: &Arc<NotToken>,
) -> Result<()> {
    let filtered: HashSet<String> = accounts
        .into_iter()
        .filter_map(|pubk| {
            match rpc_client
                .get_token_account_with_commitment(&pubk, CommitmentConfig::finalized())
                .ok()
            {
                // Return the mint account
                Some(data) => Some(data.value.unwrap().mint),
                // If the return value is None, add the public key to the Not token Hashset.
                _ => {
                    let mut nt = not_token.data.write().unwrap();
                    nt.insert(pubk);
                    None
                }
            }
        })
        .collect();

    log::info!("Filtered: {:#?}", filtered);

    Ok(())
}

#[derive(Debug)]
pub struct NotToken {
    data: RwLock<HashSet<Pubkey>>,
}

impl NotToken {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashSet::new()),
        }
    }
}
