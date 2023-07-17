#![allow(dead_code)]
use bincode::serialize;
use serde::Serialize;
use std::{
    env,
    fs::{self, File, OpenOptions},
    os::unix::prelude::FileExt,
    path::Path,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Serialize)]
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

const FILE_SYSTEM_SIZE: u32 = 1000;
const NUMBER_INODES: u32 = 200;
const BLOCK_SIZE: u64 = 512;

const INODE_PER_BLOCK: u64 = BLOCK_SIZE / std::mem::size_of::<INode>() as u64;

fn write_super_block(file: &mut File, offset: u64, data: &SuperBlock) {
    let serialized = serialize(&data).unwrap();
    file.write_all_at(&serialized, BLOCK_SIZE * offset).unwrap();
}

fn open_file(path: &String) -> File {
    // Delete if file already exists
    let path = Path::new(path);
    if path.exists() {
        fs::remove_file(path).expect("Could not delete file");
    }

    return OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(path)
        .expect("Failed to open image file");
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 1 {
        eprintln!("mkfs requires at least 2 arguments, image of the file system and files...");
        return;
    }

    let path = &args[1];
    let mut file = open_file(path);

    let super_block = SuperBlock {
        size: FILE_SYSTEM_SIZE,
        number_inodes: NUMBER_INODES,
        ..Default::default()
    };

    write_super_block(&mut file, 1, &super_block);
}
