use std::mem::size_of;

use serde::Serialize;

#[repr(u8)]
#[derive(Default, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum INodeType {
    #[default]
    FREE = 0,
    FILE = 1,
    DIRECTORY = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct SuperBlock {
    pub size: u32,
    pub number_blocks: u32,
    pub number_inodes: u32,
    pub number_logs: u32,
    pub log_start_address: u32,
    pub inode_start_address: u32,
    pub bitmap_start_address: u32,
}

#[repr(C)]
#[derive(Default, Serialize, Debug, Clone, Copy)]
pub struct INode {
    pub _type: INodeType,
    pub major: u8,
    pub minor: u8,
    pub number_links: u8,
    pub size: u32,
    // Data is broken into 12 direct blocks + 1 indirect block, just like it is done in memory
    // with paging, that is, multiple levels of redirection before you get to the data.
    pub data: [u32; DIRECT_DATA_ADDRESS_SIZE + 1],
}

#[repr(C)]
#[derive(Default, Serialize, Debug, Clone, Copy)]
pub struct DirectoryEntry {
    pub inode_number: u32,
    pub name: [u8; DIRECTORY_NAME_SIZE as usize],
}

pub const ZERO_DATA: [u8; BLOCK_SIZE as usize] = [0; BLOCK_SIZE as usize];

// General FS Information
pub const BLOCK_SIZE: u32 = 512;
pub const FILE_SYSTEM_SIZE: u32 = 1000;
pub const NUMBER_META_BLOCKS: u32 = 2 + NUMBER_LOGS + NUMBER_INODE_BLOCKS + NUMBER_BITMAPS;
pub const NUMBER_BLOCKS: u32 = FILE_SYSTEM_SIZE - NUMBER_META_BLOCKS;
pub const MAX_FILE_BLOCK: u32 = (DIRECT_DATA_ADDRESS_SIZE + INDIRECT_DATA_ADDRESS_SIZE) as u32;

// INodes
pub const NUMBER_INODES: u32 = 200;
pub const DIRECT_DATA_ADDRESS_SIZE: usize = 12;
pub const INDIRECT_DATA_ADDRESS_SIZE: usize = BLOCK_SIZE as usize / size_of::<u32>();
pub const INODE_SIZE: u32 = size_of::<INode>() as u32;
pub const INODE_PER_BLOCK: u32 = BLOCK_SIZE / INODE_SIZE;
pub const NUMBER_INODE_BLOCKS: u32 = NUMBER_INODES / INODE_PER_BLOCK + 1; // Number of blocks required to house all inodes

// Logs
pub const NUMBER_LOGS: u32 = 30;

// Free-Space Bit Map
pub const NUMBER_BITMAPS: u32 = FILE_SYSTEM_SIZE / (8 * BLOCK_SIZE) + 1; // Amount of free-space bit maps required to fully map used blocks (at least 1 block)

// Directories
pub const DIRECTORY_NAME_SIZE: u32 = 20;

pub static SUPER_BLOCK: SuperBlock = SuperBlock {
    size: FILE_SYSTEM_SIZE,
    number_inodes: NUMBER_INODES,
    number_logs: NUMBER_LOGS,
    number_blocks: NUMBER_BLOCKS,
    log_start_address: 2,
    inode_start_address: 2 + NUMBER_LOGS,
    bitmap_start_address: 2 + NUMBER_LOGS + NUMBER_INODE_BLOCKS,
};
