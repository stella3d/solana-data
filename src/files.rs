use std::{fs, path::Path};

use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;



pub fn write_json_encoded_block(slot: u64, block: &EncodedConfirmedBlock) {
    let json_r = serde_json::to_string(&block);
    match json_r {
        Ok(data) => {
            let name = slot_file_name("blocks/json", slot, JSON_EXT);

            if Path::exists(Path::new(&name)) {
                println!("\nFILE {} ALREADY PRESENT, not overriding", name);
            } else {
                println!("\nwriting to file:  {}", name);
                let _ = fs::write(name, data);
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


const JSON_EXT: &str = ".json";
const SLOT_PREFIX: &str = "slot_";
fn slot_file_name(dir: &str, slot: u64, extension: &str) -> String {
    format!("./{}/{}{}{}", dir, SLOT_PREFIX, slot, extension)
}

