use client::get_client;
use solana_sdk::account::Account;
use solana_transaction_status::{UiTransactionEncoding, EncodedConfirmedBlock};

pub mod client;
pub mod files;


fn test_client() {


    let mut client = get_client("");

    let slot_res= client.rpc.get_slot();
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

    let get_block_res = client.rpc.get_block_with_encoding(slot, UiTransactionEncoding::Base64);

    let slot_count = 8;
    let slots = client.rpc.get_blocks_with_limit(slot - slot_count, 32).unwrap();
    println!("\n{} slots to request...\n", slots.len());

    client.get_block_details(&slots,
|(slot, ecb)| {
            match *ecb {
                Some(b) => {
                    //println!("write range block file:  slot_{}", slot);
                    files::write_json_encoded_block(*slot, b);
                },
                None => {},
            }
        }); 

    match get_block_res {
        Ok(b) => {
            files::write_json_encoded_block(slot, &b);

            println!("slot:  {}", slot);
            println!("time:  {:?} , hash:  {:?}", b.block_time, b.blockhash);
            println!("rewards:\n{:?}\n", b.rewards);
            println!("TRANSACTIONS:\n");

            client.decode_txs(b.transactions);

            client.txs.iter().for_each(|tx|{
                //println!("\nTX (decoded):\n{:?}\n", tx);
            });

            if client.all_tx_accounts() > 0 {
                client.tx_accounts.iter().for_each(|a| {
                    return;
                    let owner_str = a.owner.to_string();
                    if a.data.len() <= 0  {
                        if owner_str != SPECIAL_OWNERS[0] {
                            println!("\nEMPTY account:\n  {:?}", a);
                        }
                    }
                    else if owner_str.ends_with("1111") {
                        match owner_str {
                            o if o.starts_with("Vote") => {
                                println!("\nVote account:\n  {:?}", a);
                            },
                            o if o.starts_with("Sysvar") => {
                                println!("\nSysvar account:\n  {:?}", a);
                            },
                            o if o.starts_with("NativeLoader") => {
                                println!("\nNativeLoader account:\n  {:?}", a);
                            },
                            o if o.starts_with("BPFLoader") => {
                                println!("\nBPFLoader account:\n  {:?}", a);
                            },
                            _ => {}
                        }
                    }
                    else {
                        match a {
                            Account { executable: true, .. } => {
                                println!("\nEXECUTABLE account:\n  {:?}", a);
                            },
                            _ => {},
                        }
                    }
                })
            }
        },
        Err(e) => { eprintln!("{}", e); }
    }; 
}

static SPECIAL_EXECUTABLE_OWNERS: [&str; 3] = 
[
    "NativeLoader1111111111111111111111111111111",
    "BPFLoaderUpgradeab1e11111111111111111111111",
    "BPFLoader2111111111111111111111111111111111",
];

static SPECIAL_OWNERS: [&str; 3] = 
[
    "11111111111111111111111111111111",
    "Sysvar1111111111111111111111111111111111111",
    "Vote111111111111111111111111111111111111111"
];


fn main() {
    println!("\nStarting Solana RPC client test\n");

    test_client();
}
