use client::get_client;
use solana_transaction_status::UiTransactionEncoding;

use crate::client::get_tx_accounts;

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

    let get_block_res = rpc.get_block_with_encoding(slot, UiTransactionEncoding::Base58);
    match get_block_res {
        Ok(b) => {
            println!("slot:  {}", slot);
            println!("time:  {:?} , hash:  {:?}", b.block_time, b.blockhash);
            println!("rewards:\n{:?}\n", b.rewards);

            println!("TRANSACTIONS:\n");
            let tx_len = b.transactions.len();
            let begin_get_accts = tx_len - 4;
            let mut idx = 0;

            b.transactions.iter().for_each(|tx| {
                match tx.transaction.decode() {
                    Some(decoded_tx) => {
                        println!("\nTX (DECODED):\n{:?}\n", decoded_tx);

                        if idx > begin_get_accts {
                            let tx_accts = get_tx_accounts(&rpc, decoded_tx);
                            match tx_accts {
                                Ok(txa) => println!("\ntx accounts:\n{:?}\n", txa),
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                    },
                    None => println!("\nTX:\n{:?}\n", tx.transaction),
                }
                idx += 1;
            });
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
