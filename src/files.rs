use std::{fs, path::Path};

use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

pub fn write_json_encoded_block(slot: u64, block: &EncodedConfirmedBlock) {
    let str_r = serde_json::to_string(&block);
    match str_r {
        Ok(json) => {
            let name = slot_file_name("blocks", slot, BLOCK_EXT_JSON);

            if Path::exists(Path::new(&name)) {
                println!("\nFILE {} ALREADY PRESENT, not overriding", name);
            } else {
                println!("\nwriting json to file:  {}", name);
                let _ = fs::write(name, json);
            }
        },
        Err(e) => eprintln!("{}", e),
    }
}

pub fn load_json_encoded_block(path: &str) -> Option<EncodedConfirmedBlock> {
    match fs::read(path) {
        Ok(data) => {
            match serde_json::from_slice::<EncodedConfirmedBlock>(&data) {
                Ok(block) => Some(block),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}


//const BLOCK_EXT: &str = ".block";
const BLOCK_EXT_JSON: &str = ".block.json";
const SLOT_PREFIX: &str = "slot_";
fn slot_file_name(dir: &str, slot: u64, extension: &str) -> String {
    format!("./{}/{}{}{}", dir, SLOT_PREFIX, slot, extension)
}