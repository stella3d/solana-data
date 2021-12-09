use std::{fs::{File, self}, path::Path, io};
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use zip;

use crate::{files::BLOCKS_DIR, util::{timer, log_err, timer_log_ms}};


const TEST_UNZIP_DIR: &str = "test_unzip";

pub fn extract<P: AsRef<Path>>(path: P) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { log_err(e); return; }
    };

    let mut archive = zip::ZipArchive::new(file).unwrap();

    match archive.extract(TEST_UNZIP_DIR) {
        Ok(_) => { println!("archive extraction successful") },
        Err(e) => log_err(e),
    }
}

pub fn test_extract_blocks_zip() {
    let file_path = Path::new("blocks").join("blocks_json.zip");
    println!("extracting .zip:    {}", file_path.to_str().unwrap());

    timer_log_ms(".zip extract", || {
        extract(file_path);
    });
}