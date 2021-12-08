use std::{thread, time::Duration, collections::HashMap, ops::Deref, path::{Path, PathBuf}, cmp::min};

use rayon::iter::{ParallelIterator, IntoParallelRefIterator, IntoParallelIterator};
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::{EncodedConfirmedBlock, EncodedTransactionWithStatusMeta};

use crate::files::{load_block_json, load_block_json_unwrap};


pub fn process_block_stream(block_files: &[PathBuf]) {
    println!("testing chunked stream processing...");

    let acct_set = find_account_set_stream(block_files);

    acct_set.iter().for_each(|t| {
        if *t.1 > 1 {
            println!("public key: {} , entries: {}", t.0, t.1);
        }
    });

    println!("\npublic key entries: {}", acct_set.len());
}


pub fn process_blocks(blocks: &[EncodedConfirmedBlock]) {

    let acct_set = find_account_set(blocks);

    thread::sleep(Duration::from_secs(120));

    let single_tx_blocks: Vec<&EncodedConfirmedBlock> =
        blocks.par_iter().filter_map(|b| {
            if b.transactions.len() == 1 { Some(b) } 
            else { None }
        }).collect();

    println!("\nblocks with a single transaction (out of {}):  {}\n", blocks.len(), single_tx_blocks.len());

    thread::sleep(Duration::from_secs(2));
    
    //println!("\nprinting UI inner instructions for {} blocks:\n", blocks.len());
    blocks.par_iter().for_each(|b: &EncodedConfirmedBlock| {
        if b.transactions.len() <= 1 { return; }
        let time = b.block_time.unwrap_or_default();

        let height = if let Some(h) = b.block_height { h } else { 0 };
        let height_str = if height == 0 { "none".to_string() } else { height.to_string() };

        println!("block {}: height {}, {} txs @ {}", b.blockhash, height_str, b.transactions.len(), time);

        return;
        b.transactions.iter().for_each(|tx_meta| {
            match &tx_meta.meta {
                Some(m) => {
                    match &m.inner_instructions {
                        Some(instructions) => {
                            if instructions.len() > 0 {
                                println!("\n{} UI inner instructions:", instructions.len());
                                instructions.iter().for_each(|i| println!("    {:?}", i));
                            }
                        },
                        None => todo!(),
                    }
                },
                None => todo!(),
            };
        })
    });
}

pub fn decode_txs_map(e_txs: &Vec<EncodedTransactionWithStatusMeta>) -> Vec<Transaction> {
    e_txs.iter().filter_map(|etx| {
        if let Some(tx) = etx.transaction.decode() {
            return Some(tx)
        } else {
            return None
        }
    }).collect::<Vec<Transaction>>()
}


pub fn stream_process_files<T, C: Send>(paths: &[PathBuf], 
    load_file: fn(&PathBuf) -> T, 
    each_chunk: fn(&[T]) -> C, 
    reduce: fn(Vec<C>) -> C) 
    -> C
{
    let chunk_size = min(1024, paths.len() / 32);
    let path_chunks: Vec<&[PathBuf]> = paths.chunks(chunk_size).collect();
    let intermediates: Vec<C> = path_chunks.par_iter().map(|&chunk| {
        let typed: Vec<T> = (chunk).iter().map(load_file).collect();
        each_chunk(typed.as_slice())
    }).collect();

    reduce(intermediates)
}

pub fn find_account_set_stream(block_files: &[PathBuf]) -> HashMap<Pubkey, u32> {
    stream_process_files::<EncodedConfirmedBlock, HashMap<Pubkey, u32>>(block_files, 
    |p| { load_block_json_unwrap(p) },
    |chunk| {
        let mut hash_map: HashMap<Pubkey, u32> = HashMap::<Pubkey, u32>::new();
        chunk.iter().for_each(|ecb| {
            let txs = decode_txs_map(&ecb.transactions);
            for tx in txs {
                for acct in &tx.message.account_keys {
                    match hash_map.get_mut(acct) {
                        Some(tx_count) => *tx_count = *tx_count + 1,
                        None => { hash_map.insert(*acct, 1); },
                    };
                }
            };
        });
        hash_map
    }, |sub_sets| {
        let mut outer_set = HashMap::<Pubkey, u32>::new();

        sub_sets.iter().for_each(|m| {
            println!("chunk hashmap: {} entries", m.len());
            m.iter().for_each(|entry| {
                match outer_set.get_mut(entry.0) {
                    Some(tx_count) => *tx_count = *tx_count + 1,
                    None => { outer_set.insert(*entry.0, 1); },
                };
            })
        });

        outer_set
    })
}


pub fn find_account_set(blocks: &[EncodedConfirmedBlock]) -> HashMap<Pubkey, u32> {
    let chunks: Vec<&[EncodedConfirmedBlock]> = blocks.chunks(blocks.len() / 32).collect();

    let maps: Vec<HashMap<Pubkey, u32>> = chunks.par_iter()
        .map(|&chunk| {
            let mut hash_map: HashMap<Pubkey, u32> = HashMap::<Pubkey, u32>::new();
            chunk.iter().for_each(|ecb| {
                let txs = decode_txs_map(&ecb.transactions);
                for tx in txs {
                    for acct in &tx.message.account_keys {
                        match hash_map.get_mut(acct) {
                            Some(tx_count) => *tx_count = *tx_count + 1,
                            None => { hash_map.insert(*acct, 1); },
                        };
                    }
                };
            });
            hash_map
        }).collect();


    let mut outer_set = HashMap::<Pubkey, u32>::new();

    maps.iter().for_each(|m| {
        println!("chunk hashmap: {} entries", m.len());

        m.iter().for_each(|entry| {
            match outer_set.get_mut(entry.0) {
                Some(tx_count) => *tx_count = *tx_count + 1,
                None => { outer_set.insert(*entry.0, 1); },
            };
        })
    });

    outer_set.iter().for_each(|t| {
        if *t.1 > 1 {
            println!("public key: {} , entries: {}", t.0, t.1);
        }
    });

    println!("\npublic key entries: {}", outer_set.len());

    outer_set
}