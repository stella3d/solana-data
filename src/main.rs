use std::{time::{Duration, Instant}, thread, fs, cmp::max};

use client::{get_client, MAINNET_RPC, DEVNET_RPC, TESTNET_RPC};
use serde::{Deserialize, Serialize};
use solana_program::clock::Slot;

use crate::{
    files::{test_block_loads, test_size_average, chunk_existing_blocks, CHUNKED_BLOCKS_DIR, copy_sample, BLOCK_SAMPLE_DIR, BLOCKS_DIR}, 
    util::{duration_from_hours, log_err}
};

pub mod client;
pub mod util;
pub mod analyze;
pub mod files;


#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct ScrapeState {
    pub last_slot: Slot,
}

impl Default for ScrapeState {
    fn default() -> Self {
        Self { last_slot: Default::default() }
    }
}

fn scrape_loop(duration: Duration, rpc_url: &str) {
    let r = rpc_url.clone();
    let task = || { do_scrape(rpc_url) };
    loop_task(duration, task);
}

const STATE_FILE: &str = "scrape_state.json";

fn save_state(state: ScrapeState) {
    fs::write(STATE_FILE, serde_json::to_vec(&state).unwrap()).unwrap();
}

fn load_state() -> Result<ScrapeState, serde_json::Error> {
    match fs::read(STATE_FILE) {
        Ok(data) => serde_json::from_str::<ScrapeState>
                            (std::str::from_utf8(&data).unwrap()),
        Err(_) => Ok(ScrapeState { last_slot: 0 }),
    }
}

fn do_scrape(rpc_url: &str) {
    let prev_state = load_state();
    println!("\nDO LOOP");
    println!("\nloaded previous run's state from file:\n{:?}", prev_state);

    match prev_state {
        Ok(s) => {
            if let Some(new_state) = scrape_blocks(s, rpc_url) {
                save_state(new_state); 
            };
        },
        Err(e) => log_err(&e)
    }
}


fn scrape_blocks(previous_state: ScrapeState, rpc_url: &str) -> Option<ScrapeState> {
    println!("using rpc url:  {}\n", rpc_url);
    let mut client = get_client(rpc_url);

    let slot_res= client.rpc.get_slot();
    let slot = match slot_res {
        Ok(s) => s,
        Err(e) => { 
            eprintln!("{}", e);
            0u64 
        }
    };
    if slot <= 0 {
        return None; 
    }

    let backlog_limit: usize = 4096;
    let slot_count = max(slot - previous_state.last_slot, backlog_limit as u64);
    let slots_r = client.rpc.get_blocks_with_limit(slot_count, backlog_limit);
    let slots = match slots_r {
        Ok(s) => s,
        Err(_) => vec![],
    };

    println!("\n{} slots to request: {}-{}\n", slots.len(), slots.first().unwrap(), slots.last().unwrap());

    let last = client.get_block_details(&slots,
|(slot, ecb)| {
            match *ecb {
                Some(b) => {
                    //println!("write range block file:  slot_{}", slot);
                    files::write_json_encoded_block(*slot, b);
                },
                None => {},
            }
        });

    if last == 0 { None }
    else { Some(ScrapeState { last_slot: last }) }
}

fn loop_task<F: Fn() -> ()>(total_time: Duration, loop_fn: F) {
    let start = Instant::now();
    let end = start + total_time;
    while Instant::now() < end {
        loop_fn();
    }
    println!("loop task finished after: {} milliseconds", start.elapsed().as_millis());
}


fn main() {
    println!("\nStarting Solana RPC client test\n");

    //chunk_existing_blocks(80);
    //thread::sleep(Duration::from_secs(15)); 
     
    //copy_sample(BLOCKS_DIR, 50);
    //thread::sleep(Duration::from_secs(600));

    /* 
    test_block_loads();
    thread::sleep(Duration::from_secs(180));

    test_size_average(CHUNKED_BLOCKS_DIR);
    thread::sleep(Duration::from_secs(600));
    */

    let rpc = MAINNET_RPC;
    scrape_loop(duration_from_hours(4), &rpc);
}
