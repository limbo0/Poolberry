use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
// ********************************************************************************************
#[derive(Debug, Default)]
pub struct MintAccounts {
    pub data: RwLock<BTreeMap<Pubkey, usize>>,
}

impl MintAccounts {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(BTreeMap::new()),
        }
    }

    // Does a batch update, for every insert or update (1 value) is added to the token counter.
    pub fn update_or_insert(&self, mint_accounts: HashSet<Pubkey>) {
        mint_accounts
            .into_iter()
            .map(|mint_account| {
                if let Some(value) = self.data.try_write().unwrap().get_mut(&mint_account) {
                    // If the address(key) is present then it should only update the counter of that key.
                    log::info!("Mint account exists, updating its current count!!");
                    *value += 1usize;
                } else {
                    // Or else insert a new key(address) value(mint_account)
                    log::info!("Mint account does not exists, adding now!!");
                    self.data.try_write().unwrap().insert(mint_account, 1usize);
                }
            })
            .collect::<()>()
    }

    // convert to vec and and sory descending
    pub fn sort_descending(&self) -> Result<Vec<(Pubkey, usize)>> {
        let mut entries: Vec<(Pubkey, usize)> =
            self.data.try_read().unwrap().clone().into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(entries)
    }

    pub fn contains_key(&self, address: &Pubkey) -> bool {
        self.data.try_read().unwrap().contains_key(address)
    }

    pub fn total_length(&self) -> Result<Option<usize>> {
        match self.data.try_read().ok() {
            Some(data) => Ok(Some(data.len())),
            None => Ok(None),
        }
    }
}

// ********************************************************************************************
#[derive(Debug, Default)]
pub struct NotToken {
    data: RwLock<HashSet<Pubkey>>,
}

impl NotToken {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashSet::new()),
        }
    }

    // Helper read function.
    pub fn read_data(&self) -> Result<Option<RwLockReadGuard<HashSet<Pubkey>>>> {
        Ok(Some(self.data.try_read().unwrap()))
    }

    // Helper write function.
    pub fn write_data(&self) -> Result<RwLockWriteGuard<HashSet<Pubkey>>> {
        Ok(self.data.try_write().unwrap())
    }

    // Will return None if any error occours.
    pub fn total_length(&self) -> Result<Option<usize>> {
        match self.data.try_read().ok() {
            Some(data) => Ok(Some(data.len())),
            None => Ok(None),
        }
    }

    pub fn contains(&self, address: &Pubkey) -> bool {
        self.data.try_read().unwrap().contains(address)
    }
}
