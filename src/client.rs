use std::{collections::{HashSet}, sync::Arc, path::Path};

use solana_client::{self, rpc_client::RpcClient, client_error::{ClientError}, rpc_response::RpcBlockProduction};
use solana_program::{pubkey::Pubkey, clock::Slot};
use solana_sdk::{transaction::Transaction, account::Account};
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding, EncodedConfirmedBlock};

use crate::{files::{slot_json_path}, networks::DEVNET_RPC};


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
                //for a in av { println!("\naccount:    {:?}\n", a); }
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

    pub fn get_block_details(&mut self, slots: &Vec<Slot>, callback: fn(&(Slot, Option<&EncodedConfirmedBlock>))) -> Slot {
        let len = (*slots).len();
        if len > 4096 || len < 1 {
            println!("only ranges 1-4096 in length supported right now, input length:  {}", len);
            return 0;
        }

        for s in slots {
            if Path::new(&slot_json_path(*s)).exists() { 
            println!("SKIP: request slot {}, file exists", s);
                continue; 
            }

            print!("requesting slot {}...\t", s);
            let r = self.rpc.get_block_with_encoding(*s, UiTransactionEncoding::Base64);
            match r {
                Ok(ecb) => {
                    let opt = Some(&ecb);
                    callback(&(*s, opt));
                },
                Err(e) => {
                    eprintln!("{}", e);
                },
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

const DEFAULT_TMP_CAPACITY: usize = 256;

pub fn get_client (rpc_url: &str) -> ClientWrapper {
    let mut rpc: &str = rpc_url;
    if rpc_url.is_empty() {
        rpc = DEVNET_RPC.clone();
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
