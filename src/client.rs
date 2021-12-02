use std::collections::HashSet;

use solana_client::{self, rpc_client::RpcClient, client_error::ClientError};
use solana_program::pubkey::Pubkey;
use solana_sdk::{transaction::Transaction, account::Account};


pub static DEVNET_RPC: &str = "https://api.devnet.solana.com";
pub static TESTNET_RPC: &str = "https://api.testnet.solana.com";
pub static MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";


pub fn get_client (rpc_url: &str) -> RpcClient {
    let mut rpc = rpc_url;
    if rpc_url.is_empty() {
        rpc = DEVNET_RPC;
    }
    RpcClient::new(rpc.to_string())
}


pub fn get_tx_accounts(rpc: &RpcClient, tx: Transaction) -> 
    Result<Vec<Option<Account>>, ClientError> 
{
    rpc.get_multiple_accounts(&tx.message.account_keys)
}

pub fn all_tx_accounts(rpc: &RpcClient, txs: &[Transaction]) -> 
    Result<Vec<Option<Account>>, ClientError> 
{
    let mut keys = HashSet::<Pubkey>::new();
    txs.iter().for_each(|tx| {
        tx.message.account_keys.iter().for_each(|pk| { 
            // TODO - combine multiple responses into one
            if keys.len() < 100 {
                keys.insert(*pk); 
            }
        })
    });

    let keys_vec: Vec<Pubkey> = keys.into_iter().collect();

    rpc.get_multiple_accounts(&keys_vec)
}
