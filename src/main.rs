use client::get_client;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::UiTransactionEncoding;

use crate::client::{get_tx_accounts, all_tx_accounts};

pub mod client;


fn test_client() {
    let rpc = get_client("");

    let slot_res= rpc.get_slot();
    let slot = match slot_res {
        Ok(slot) => slot,
        Err(e) => { 
            eprintln!("{}", e);
            0u64 
        }
    };
    if slot <= 0 {
        return; 
    }

    let get_block_res = rpc.get_block_with_encoding(slot, UiTransactionEncoding::Base64);
    match get_block_res {
        Ok(b) => {
            println!("slot:  {}", slot);
            println!("time:  {:?} , hash:  {:?}", b.block_time, b.blockhash);
            println!("rewards:\n{:?}\n", b.rewards);

            println!("TRANSACTIONS:\n");

            let decoded: Vec<Transaction> = b.transactions.iter()
                .map(|etx| {
                    etx.transaction.decode()
                })
                .filter(|opt| opt.is_some())
                .map(|opt| opt.unwrap())
                .collect();

            decoded.iter().for_each(|tx|{
                println!("\nTX (decoded):\n{:?}\n", tx);
            });

            let all_tx_accts = all_tx_accounts(&rpc, &decoded);
            match all_tx_accts {
                Ok(accts) => {
                    //println!("\nall tx accounts: {:?}\n", accts)
                    accts.iter().for_each(|a| {
                        match a {
                            Some(val) => println!("\nacct: {:?}\n", *val),
                            None => {},
                        }
                    });
                },
                Err(e) => eprintln!("{}", e),
            }
        },
        Err(e) => { 
            eprintln!("{}", e);
        }
        
    }; 

}

fn main() {
    println!("\nStarting Solana RPC client test\n");

    test_client();
}
