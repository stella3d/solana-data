use client::get_client;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::UiTransactionEncoding;

use crate::client::{ClientWrapper};

pub mod client;


fn test_client() {
    let mut rpc_wrapper = get_client("");

    let slot_res= rpc_wrapper.rpc.get_slot();
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

    let get_block_res = rpc_wrapper.rpc.get_block_with_encoding(slot, UiTransactionEncoding::Base64);
    match get_block_res {
        Ok(b) => {
            println!("slot:  {}", slot);
            println!("time:  {:?} , hash:  {:?}", b.block_time, b.blockhash);
            println!("rewards:\n{:?}\n", b.rewards);

            println!("TRANSACTIONS:\n");

            rpc_wrapper.decode_txs(b.transactions);

            rpc_wrapper.txs.iter().for_each(|tx|{
                println!("\nTX (decoded):\n{:?}\n", tx);
            });

            if rpc_wrapper.all_tx_accounts() > 0 {
                rpc_wrapper.tx_accounts.iter().for_each(|a| {
                    println!("\nacct: {:?}", a);
                })
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
