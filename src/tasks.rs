pub(crate) const CHUNK_BLOCKS_TASK: &str = "chunk_blocks";
pub(crate) const COUNT_KEY_TXS_TASK: &str = "count_txs";
pub(crate) const MEAN_FILE_SIZE_TASK: &str = "mean_fsize";
pub(crate) const SCRAPE_BLOCKS_TASK: &str = "scrape_blocks";
pub(crate) const COMPARE_BLOCK_LOADS_TASK: &str = "cmp_block_loads";
pub(crate) const BLOCK_SAMPLE_TASK: &str = "block_sample";
pub(crate) const GET_BLOCK_PROD_TASK: &str = "get_prod";
pub(crate) const SUB_BASIC_TASK: &str = "sub_basic";

pub(crate) static TASK_NAMES: [&str; 8] = [
    CHUNK_BLOCKS_TASK, COUNT_KEY_TXS_TASK, MEAN_FILE_SIZE_TASK,
    SCRAPE_BLOCKS_TASK, COMPARE_BLOCK_LOADS_TASK, BLOCK_SAMPLE_TASK,
    GET_BLOCK_PROD_TASK, SUB_BASIC_TASK
];