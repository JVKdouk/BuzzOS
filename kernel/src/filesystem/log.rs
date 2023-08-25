use crate::{memory::mem::mem_move, sync::spin_mutex::SpinMutex};

use super::{
    cache::{read_disk_block, write_disk_block, CacheBlock},
    fs::{SECONDARY_BLOCK_ID, SUPER_BLOCK_CACHE},
    ide::BLOCK_SIZE,
};

const LOG_BLOCK_SIZE: usize = 30;

#[repr(C)]
struct DiskLogHeader {
    count: u32,                    // Number of blocks to commit
    blocks: [u32; LOG_BLOCK_SIZE], // Blocks to commit
}

#[repr(C)]
struct DiskLog {
    start: u32,
    size: u32,
    outstanding: u32, // Number of FS calls currently running (no commit can happen while this is not zero)
    commiting: bool,  // Status of the log commit
    dev: u32,
    header: DiskLogHeader,
}

impl DiskLog {
    pub const fn new() -> Self {
        DiskLog {
            start: 0,
            size: 0,
            outstanding: 0,
            commiting: false,
            dev: 0,
            header: DiskLogHeader {
                count: 0,
                blocks: [0; LOG_BLOCK_SIZE],
            },
        }
    }
}

static DISK_LOG: SpinMutex<DiskLog> = SpinMutex::new(DiskLog::new());

pub fn setup_log() {
    let mut disk_log = DISK_LOG.lock();
    let super_block = SUPER_BLOCK_CACHE.lock();

    disk_log.dev = SECONDARY_BLOCK_ID;
    disk_log.size = super_block.number_logs;
    disk_log.start = super_block.log_start_address;
}

pub fn read_log_header() {
    let mut disk_log = DISK_LOG.lock();

    let in_disk_log_header = read_disk_block(disk_log.dev, disk_log.start);
    let mut header_data_lock = in_disk_log_header.lock();
    let header_data = header_data_lock.cast_to::<DiskLogHeader>();
    disk_log.header.count = header_data[0].count;

    for i in 0..disk_log.header.count {
        disk_log.header.blocks[i as usize] = header_data[0].blocks[i as usize];
    }
}

pub fn write_log_header() {
    let disk_log = DISK_LOG.lock();
    let in_disk_log_header = read_disk_block(disk_log.dev, disk_log.start);
    let mut header_data_lock = in_disk_log_header.lock();
    let header_data = header_data_lock.cast_to::<DiskLogHeader>();
    header_data[0].count = disk_log.header.count;

    for i in 0..disk_log.header.count {
        header_data[0].blocks[i as usize] = disk_log.header.blocks[i as usize];
    }

    unsafe { in_disk_log_header.force_unlock() };

    // Block already in cache, skips Rust borrow-checker
    let new_block = read_disk_block(disk_log.dev, disk_log.start);
    write_disk_block(new_block);
}

pub fn recover_log() {
    read_log_header();
    write_log_blocks();
    DISK_LOG.lock().header.count = 0;
}

/// Write log blocks back to disk
pub fn write_log_blocks() {
    let disk_log = DISK_LOG.lock();

    for (i, block_number) in disk_log.header.blocks.iter().enumerate() {
        let log_block = read_disk_block(disk_log.dev, disk_log.start + i as u32 + 1);
        let disk_block = read_disk_block(disk_log.dev, *block_number);
        let log_block_ptr = log_block.lock().data.as_mut_ptr();
        let disk_block_ptr = disk_block.lock().data.as_mut_ptr();
        unsafe { mem_move(log_block_ptr, disk_block_ptr, BLOCK_SIZE) };
        write_disk_block(disk_block);
    }
}

/// Moves blocks from cache to log
pub fn move_cache_to_log() {
    let disk_log = DISK_LOG.lock();

    for (i, block_number) in disk_log.header.blocks.iter().enumerate() {
        let to_block = read_disk_block(disk_log.dev, disk_log.start + i as u32 + 1);
        let from_block = read_disk_block(disk_log.dev, *block_number);
        let to_block_ptr = to_block.lock().data.as_mut_ptr();
        let from_block_ptr = from_block.lock().data.as_mut_ptr();
        unsafe { mem_move(to_block_ptr, from_block_ptr, BLOCK_SIZE) };
        write_disk_block(from_block);
    }
}

pub fn commit_log() {
    let mut disk_log = DISK_LOG.lock();

    if disk_log.header.count > 0 {
        move_cache_to_log();
        write_log_header();
        write_log_blocks();
        disk_log.header.count = 0;
        write_log_header();
    }
}

pub fn write_block_log(block: CacheBlock) {
    let mut disk_log = DISK_LOG.lock();

    if disk_log.header.count >= LOG_BLOCK_SIZE as u32 || disk_log.header.count >= disk_log.size - 1
    {
        panic!("[ERROR] FS Log is too big");
    }

    let mut index = 0;
    for block_number in disk_log.header.blocks {
        if block_number == block.lock().block_number {
            break;
        }

        index += 1;
    }

    if index == disk_log.header.count {
        disk_log.header.count += 1;
    }

    block.lock().dirty = true;
}
