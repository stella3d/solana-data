use std::{collections::{HashSet}, sync::Arc, path::Path};

use solana_client::{self, rpc_client::RpcClient, client_error::{ClientError}, rpc_response::RpcBlockProduction};
use solana_program::{pubkey::Pubkey, clock::Slot};
use solana_sdk::{transaction::Transaction, account::Account};
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding, EncodedConfirmedBlock};

use crate::{files::{slot_json_path}, networks::DEVNET_RPC, util::log_err};


// TODO - basic comments explaining why the rpc wrapper etc
#[derive(Clone)]
pub struct SolClient {
    pub rpc: Arc<RpcClient>,

    pub t_key_set: HashSet<Pubkey>,
    pub t_key_vec: Vec<Pubkey>,

    pub tx_accounts: Vec::<Account>,
    pub txs: Vec::<Transaction>,
}

impl SolClient {
    const TMP_BUFFER_LEN: usize = 256;

    pub fn get (rpc_url: &str) -> SolClient {
        let mut rpc: &str = rpc_url;
        if rpc_url.is_empty() { rpc = DEVNET_RPC; }
    
        SolClient { 
            rpc: Arc::new(RpcClient::new(rpc.to_string())), 
            t_key_set: HashSet::<Pubkey>::with_capacity(Self::TMP_BUFFER_LEN), 
            t_key_vec: Vec::<Pubkey>::with_capacity(Self::TMP_BUFFER_LEN), 
            tx_accounts: Vec::<Account>::with_capacity(Self::TMP_BUFFER_LEN),
            txs: Vec::<Transaction>::with_capacity(Self::TMP_BUFFER_LEN) 
        }
    }

    pub fn get_accounts(&mut self) -> Option<&Vec<Account>> {
        self.tx_accounts.clear();

        // only 100 accts allowed per RPC request
        let chunks = self.t_key_vec.chunks(100);
        chunks.for_each(|c| {
            let c_accts = self.rpc.get_multiple_accounts(&c);
            match c_accts {
                Ok(accts) => {
                    for ao in accts {
                        match ao {
                            Some(a) => self.tx_accounts.push(a.clone()),
                            None => {},
                        }
                    }
                },
                Err(e) => eprintln!("{}", e),
            }
        });

        if self.tx_accounts.len() > 0 { Some(&self.tx_accounts) } 
        else { None }
    }

    pub fn get_tx_accounts(rpc: &RpcClient, tx: Transaction) -> 
        Result<Vec<Option<Account>>, ClientError> 
    {
        rpc.get_multiple_accounts(&tx.message.account_keys)
    }

    pub fn decode_txs(&mut self, e_txs: Vec<EncodedTransactionWithStatusMeta>) {
        self.txs.clear();
        e_txs.iter().for_each(|etx| {
            match etx.transaction.decode() {
                Some(tx) => self.txs.push(tx), 
                None => {}
            }
        });
    }

    pub fn get_block_details(&mut self, slots: &Vec<Slot>, callback: fn(&(Slot, Option<&EncodedConfirmedBlock>))) -> Slot {
        for s in slots {
            if Path::new(&slot_json_path(*s)).exists() { 
                println!("skipping request for slot {}: file exists", s);
                continue; 
            }
            match self.rpc.get_block_with_encoding(*s, UiTransactionEncoding::Base64) {
                Ok(ecb) => {
                    callback(& (*s, Some(&ecb)));
                },
                Err(e) => log_err(&e),
            }
        }

        *slots.last().unwrap()
    }

    pub fn get_block_production(&self) -> Result<RpcBlockProduction, ClientError> {
        match self.rpc.get_block_production() {
            Ok(response) => Ok(response.value),
            Err(e) => Err(e) 
        }
    }
}


// TODO - put in the impl or delete
pub fn get_tx_accounts(rpc: &RpcClient, tx: Transaction) -> 
    Result<Vec<Option<Account>>, ClientError> 
{
    rpc.get_multiple_accounts(&tx.message.account_keys)
}
