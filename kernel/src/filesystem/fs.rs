use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{string::String, vec};

use crate::filesystem::log::setup_log;
use crate::{println, sync::spin_mutex::SpinMutex};

use super::{
    cache::{read_disk_block, CacheBlock},
    ide::BLOCK_SIZE,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SuperBlock {
    pub size: u32,
    pub number_blocks: u32,
    pub number_inodes: u32,
    pub number_logs: u32,
    pub log_start_address: u32,
    pub inode_start_address: u32,
    pub bitmap_start_address: u32,
}

const INODE_SIZE: usize = core::mem::size_of::<INode>();
const INODE_PER_BLOCK: usize = BLOCK_SIZE / INODE_SIZE;
const INODE_DATA_ADDRESS_SIZE: usize = 12;

const DIRECTORY_NAME_SIZE: usize = 20;

#[repr(u8)]
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum INodeType {
    #[default]
    FREE = 0,
    FILE = 1,
    DIRECTORY = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct INode {
    _type: INodeType,
    major: u8,
    minor: u8,
    number_links: u8,
    size: u32,
    data: [u32; INODE_DATA_ADDRESS_SIZE + 1],
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct DirectoryEntry {
    pub inode_number: u32,
    pub name: [u8; DIRECTORY_NAME_SIZE as usize],
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

pub const SECONDARY_BLOCK_ID: u32 = 1;
pub const ROOT_INODE_NUMBER: u32 = 1;

pub static SUPER_BLOCK_CACHE: SpinMutex<SuperBlock> = SpinMutex::new(SuperBlock::new());

pub fn load_super_block() {
    let cache_data = read_disk_block(SECONDARY_BLOCK_ID, 1).lock().data.as_ptr();
    let super_block = unsafe { *(cache_data as *const SuperBlock) };
    *SUPER_BLOCK_CACHE.lock() = super_block;
}

fn clear_block(block: CacheBlock) {
    block.lock().data = [0; BLOCK_SIZE];
}

pub fn get_inode(device: u32, inode_number: u32) -> INode {
    let inode_offset = INODE_SIZE / BLOCK_SIZE;
    let inode_start = SUPER_BLOCK_CACHE.lock().inode_start_address;
    let inode_block = inode_start + inode_number * inode_offset as u32;

    let block_data = read_disk_block(SECONDARY_BLOCK_ID, inode_block);
    let mut block_data = block_data.lock();
    let inode_list = block_data.cast_to::<INode>();
    let inode_index = inode_number as usize % INODE_PER_BLOCK;

    inode_list[inode_index]
}

pub fn read_inode_data(inode: &INode, mut offset: u32, mut length: u32) -> Vec<u8> {
    let data = inode.data;

    assert!(offset <= inode.size);

    // Truncate if length and offset are outside the side of the inode
    if offset + length > inode.size {
        length = inode.size - offset;
    }

    let mut buffer = vec![0; length as usize];
    let mut count: usize = 0;
    while count < length as usize {
        let block_offset = offset as usize % BLOCK_SIZE;
        let block_number = inode.data[offset as usize / BLOCK_SIZE];
        let block_data = read_disk_block(SECONDARY_BLOCK_ID, block_number)
            .lock()
            .data;

        let byte_count = core::cmp::min(
            length as usize - count,
            BLOCK_SIZE - offset as usize % BLOCK_SIZE,
        );

        // Copy from block to buffer
        buffer.as_mut_slice()[count..(count + byte_count)]
            .copy_from_slice(&block_data[block_offset..(block_offset + byte_count)]);

        count += byte_count;
        offset += byte_count as u32;
    }

    buffer
}

pub fn get_root_inode() -> INode {
    get_inode(SECONDARY_BLOCK_ID, ROOT_INODE_NUMBER)
}

pub fn setup_file_system() {
    load_super_block();

    println!(
        "[KERNEL] Filesystem Initialized ({} inodes)",
        SUPER_BLOCK_CACHE.lock().number_inodes
    );

    setup_log();
}

pub fn read_dir(inode: &INode) -> Option<Vec<DirectoryEntry>> {
    assert!(inode._type == INodeType::DIRECTORY);

    // Empty directory
    if inode.size == 0 {
        return None;
    }

    let dir_data = read_inode_data(inode, 0, inode.size);
    let dir_count = inode.size / core::mem::size_of::<DirectoryEntry>() as u32;

    let dir_children = unsafe {
        core::slice::from_raw_parts(
            dir_data.as_ptr() as *const DirectoryEntry,
            dir_count as usize,
        )
    };

    Some(dir_children.to_owned())
}

pub fn get_path_filename(path: String) -> String {
    path.split('/').last().unwrap().to_string()
}

pub fn find_inode_by_path(path: String) -> Option<INode> {
    let mut dirs;

    if path.chars().nth(0).unwrap() == '/' {
        dirs = path.split('/').skip(1);
    } else {
        dirs = path.split('/').skip(0);
    }

    let mut current_inode = get_root_inode();
    for dir in dirs {
        let Some(entries) = read_dir(&current_inode) else {
            return None;
        };

        // Search for directory/file in current directory
        let mut found = false;
        for entry in entries {
            let name = core::str::from_utf8(&entry.name).unwrap();
            if name.trim_matches(char::from(0)) == dir {
                current_inode = get_inode(SECONDARY_BLOCK_ID, entry.inode_number);
                found = true;
                break;
            }
        }

        if found == false {
            return None;
        }
    }

    Some(current_inode)
}
