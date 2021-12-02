use std::{collections::HashSet, slice::Chunks, sync::Arc};

use solana_client::{self, rpc_client::RpcClient, client_error::{ClientError, reqwest::Client}};
use solana_program::pubkey::Pubkey;
use solana_sdk::{transaction::Transaction, account::Account};
use solana_transaction_status::EncodedTransactionWithStatusMeta;


pub static DEVNET_RPC: &str = "https://api.devnet.solana.com";
pub static TESTNET_RPC: &str = "https://api.testnet.solana.com";
pub static MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";


const DEFAULT_TMP_CAPACITY: usize = 256;

#[derive(Clone)]
pub struct ClientWrapper {
    pub rpc: Arc<RpcClient>,

    pub t_key_set: HashSet<Pubkey>,
    pub t_key_vec: Vec<Pubkey>,

    pub tx_accounts: Vec::<Account>,
    pub txs: Vec::<Transaction>,
}

impl ClientWrapper {
    fn gather_keys<'a>(s: &'a mut ClientWrapper) -> &'a Vec<Pubkey> {
        let k_set = &mut s.t_key_set;
        k_set.clear();
        
        s.txs.iter().for_each(|tx| {
            tx.message.account_keys.iter().for_each(|pk| { 
                k_set.insert(*pk); 
            });
        });

        s.t_key_vec.clear();
        for k in k_set.iter() {
            s.t_key_vec.push(k.clone());
        }

        &s.t_key_vec
    }

    pub fn get_accounts(&mut self) -> Option<&Vec<Account>> {
        self.tx_accounts.clear();

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

    pub fn all_tx_accounts(&mut self) -> usize 
    {
        let keys = Self::gather_keys(self);

        let mut request_count = keys.len() / 100;
        if keys.len() % 100 != 0 {
            request_count += 1;
        }

        println!("account number: {}", keys.len());
        println!("request count for all txs: {}", request_count);

        match self.get_accounts() {
            Some(av) => {
                for a in av { println!("\naccount:    {:?}\n", a); }
                av.len()
            },
            None => 0
        }
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
}

fn push_fmap<S, D>(src: Vec<Option<S>>, mut dest: Vec<D>, 
    filter_map: fn(&Option<S>) -> Option<D>) 
{
    src.iter().for_each(|s| {
        match filter_map(s) {
            Some(d) => dest.push(d),
            None => {},
        }
    });
}

pub fn get_client (rpc_url: &str) -> ClientWrapper {
    let mut rpc = rpc_url;
    if rpc_url.is_empty() {
        rpc = DEVNET_RPC;
    }

    ClientWrapper { 
        rpc: Arc::new(RpcClient::new(rpc.to_string())), 
        t_key_set: HashSet::<Pubkey>::with_capacity(DEFAULT_TMP_CAPACITY), 
        t_key_vec: Vec::<Pubkey>::with_capacity(DEFAULT_TMP_CAPACITY), 
        tx_accounts: Vec::<Account>::with_capacity(DEFAULT_TMP_CAPACITY),
        txs: Vec::<Transaction>::with_capacity(DEFAULT_TMP_CAPACITY), 
    }
}


pub fn get_tx_accounts(rpc: &RpcClient, tx: Transaction) -> 
    Result<Vec<Option<Account>>, ClientError> 
{
    rpc.get_multiple_accounts(&tx.message.account_keys)
}

/*
pub fn all_tx_accounts(rpc: &RpcClient, txs: &[Transaction]) -> 
    Result<Vec<Option<Account>>, ClientError> 
{
    let mut keys = HashSet::<Pubkey>::new();
    txs.iter().for_each(|tx| {
        tx.message.account_keys.iter().for_each(|pk| { 
            // TODO - combine multiple responses into one
            keys.insert(*pk); 
        })
    });

    let mut request_count = keys.len() / 100;
    if keys.len() % 100 != 0 {
        request_count += 1;
    }

    println!("account number: {}", keys.len());
    println!("request count for all txs: {}", request_count);

    let keys_vec: Vec<Pubkey> = keys.into_iter().collect();
    // only 100 accounts allowed per request, make multiple if needed
    if keys_vec.len() > 100 {
        let mut results = Vec::<Option<Account>>::with_capacity(keys_vec.len());
        let chunks = keys_vec.chunks(100);
        chunks.for_each(|c| {
            let c_accts = rpc.get_multiple_accounts(&c);
            match c_accts {
                Ok(mut accts) => {
                    results.append(&mut accts);
                },
                Err(_) => {},
            }
        });
        Ok(results)
    }
    else {
        rpc.get_multiple_accounts(&keys_vec)
    }
}
*/
