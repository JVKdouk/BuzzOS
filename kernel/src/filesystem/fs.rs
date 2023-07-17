use crate::{println, sync::spin_mutex::SpinMutex};

use super::{
    cache::{read_disk_block, CacheBlock},
    ide::BLOCK_SIZE,
};

#[derive(Debug, Clone, Copy, Default)]
struct SuperBlock {
    size: u32,
    number_blocks: u32,
    number_inodes: u32,
    number_logs: u32,
    log_start_address: u32,
    inode_start_address: u32,
    bitmap_start_address: u32,
}

const INODE_DATA_ADDRESS_SIZE: usize = 12;
struct INode {
    _type: u8,
    major: u8,
    minor: u8,
    number_links: u8,
    size: u32,
    data: [u32; INODE_DATA_ADDRESS_SIZE + 1],
}

impl SuperBlock {
    pub const fn new() -> Self {
        SuperBlock {
            size: 0,
            number_blocks: 0,
            number_inodes: 0,
            number_logs: 0,
            log_start_address: 0,
            inode_start_address: 0,
            bitmap_start_address: 0,
        }
    }
}

/// The file system is a way of structuring data in the disk. Just like a data structure in memory,
/// a file system facilitates the storage and retrieval of organized data. File System often have
/// mechanisms for backing up data in order to recover it in case of system failure. To find more
/// information about File Systems, visit https://wiki.osdev.org/File_Systems.

const SECONDARY_BLOCK_ID: u32 = 1;

static SUPER_BLOCK_CACHE: SpinMutex<SuperBlock> = SpinMutex::new(SuperBlock::new());

pub fn load_super_block() {
    let cache_data = read_disk_block(SECONDARY_BLOCK_ID, 8).lock().data.as_ptr();
    let super_block = unsafe { *(cache_data as *const SuperBlock) };
    *SUPER_BLOCK_CACHE.lock() = super_block;
}

fn clear_block(block: CacheBlock) {
    block.lock().data = [0; BLOCK_SIZE];
}

pub fn setup_file_system() {
    load_super_block();

    println!(
        "[KERNEL] Filesystem Initialized ({} inodes)",
        SUPER_BLOCK_CACHE.lock().number_inodes
    );
}
