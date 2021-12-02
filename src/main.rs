use client::get_client;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_transaction_status::UiTransactionEncoding;

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
                    let owner_str = a.owner.to_string();
                    if a.data.len() <= 0  {
                        if owner_str != SPECIAL_OWNER_NAMES[0] {
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

static SPECIAL_OWNER_NAMES: [&str; 5] = 
[
    "11111111111111111111111111111111",
    "NativeLoader1111111111111111111111111111111",
    "BPFLoader2111111111111111111111111111111111",
    "Sysvar1111111111111111111111111111111111111",
    "Vote111111111111111111111111111111111111111"
];


fn main() {
    println!("\nStarting Solana RPC client test\n");

    test_client();
}
