use crate::common::{MintAccounts, NotToken};
use anyhow::Result;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{collections::HashSet, sync::Arc};

// ********************************************************************************************
type Myclient = std::sync::Arc<RpcClient>;
// let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<Pubkey>()?;
// let token_extension_program =
//     "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".parse::<Pubkey>()?;

// ********************************************************************************************
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
    // If transaction signature is invalid the following will handle.
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

// ********************************************************************************************
// This should receive a list of new Pubkey and a list of predefined tokens which are not tokens
// Add pkeys which are not tokens to the not token list.
pub async fn check_if_token(
    rpc_client: &Myclient,
    tx_involved_pubkeys: Vec<Pubkey>,
    not_token_hset: &Arc<NotToken>,
    mint_acc_bmap: &Arc<MintAccounts>,
) -> Result<()> {
    // Only once while initialization the true block should execute.
    match not_token_hset
        .read_data()
        .unwrap()
        .is_some_and(|x| x.is_empty())
    {
        false => {
            log::info!("Not_token_hashset is not empty!");
            log::info!("Pre count filter 1 : {:#?}", tx_involved_pubkeys.len());
            let not_exist_not_token_hashset: HashSet<Pubkey> = tx_involved_pubkeys
                .into_iter()
                .filter(|pubk| !not_token_hset.contains(pubk))
                .collect();
            log::info!(
                "Post count filter 1: {:#?}",
                not_exist_not_token_hashset.len()
            );

            // The addresses which were not present in the not_token_hashset
            // will be checked via rpc call.
            let mint_accounts: HashSet<Pubkey> = not_exist_not_token_hashset
                .into_iter()
                .filter_map(|pubk| {
                    match rpc_client
                        .get_token_account_with_commitment(&pubk, CommitmentConfig::finalized())
                        .ok()
                    {
                        // If successfull return the mint account
                        Some(data) => {
                            log::info!("Found Mint account!");
                            Some(data.value.unwrap().mint.parse::<Pubkey>().unwrap())
                        }
                        // If the return value is None, add the public key to the Not token Hashset.
                        _ => {
                            log::info!("Mint account not found! Updating the not_token_hset");
                            let mut nt = not_token_hset.write_data().unwrap();
                            nt.insert(pubk);
                            None
                        }
                    }
                })
                .collect();
            mint_acc_bmap.update_or_insert(mint_accounts);
        }
        true => {
            log::info!("Hashset is empty!");
            let mint_accounts: HashSet<Pubkey> = tx_involved_pubkeys
                .into_iter()
                .filter_map(|pubk| {
                    match rpc_client
                        .get_token_account_with_commitment(&pubk, CommitmentConfig::finalized())
                        .ok()
                    {
                        // Return the mint account
                        Some(data) => {
                            log::info!("Found Mint account!");
                            Some(data.value.unwrap().mint.parse::<Pubkey>().unwrap())
                        }
                        // If the return value is None, add the public key to the Not token Hashset.
                        _ => {
                            log::info!("Mint account not found! Updating the not_token_hset");
                            let mut nt = not_token_hset.write_data().unwrap();
                            nt.insert(pubk);
                            None
                        }
                    }
                })
                .collect();
            mint_acc_bmap.update_or_insert(mint_accounts);
            // log::info!("Mint accounts: {:?}", filtered);
        }
    }

    Ok(())
}
