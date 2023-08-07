#![allow(dead_code)]
#![feature(slice_pattern)]
#![feature(generic_arg_infer)]
use bincode::serialize;
use lazy_static::lazy_static;

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    mem::size_of,
    os::unix::prelude::FileExt,
    path::Path,
    slice::from_raw_parts,
    sync::{Mutex, MutexGuard},
};

mod defs;
use defs::*;

lazy_static! {
    static ref FILE: Mutex<File> = {
        let args: Vec<_> = env::args().collect();
        let path = &args[1];
        let file = open_file(path);
        Mutex::new(file)
    };
}

static NEXT_FREE_DATA_BLOCK: Mutex<u32> = Mutex::new(NUMBER_META_BLOCKS);

macro_rules! inode_block {
    ($number:expr) => {
        $number / INODE_PER_BLOCK + SUPER_BLOCK.inode_start_address
    };
}

fn read_inode(inode_number: u32) -> INode {
    let block_index = inode_block!(inode_number);
    let mut current_data = read_sector(block_index) as [u8; BLOCK_SIZE as usize];
    let inode_index = inode_number % INODE_PER_BLOCK;
    let inode_start_address = (inode_index * INODE_SIZE) as usize;
    let inode_end_address = inode_start_address + INODE_SIZE as usize;

    // Take slice from data array
    let current_inode_data: &mut [u8; INODE_SIZE as usize] = current_data
        [inode_start_address..inode_end_address]
        .as_mut()
        .try_into()
        .unwrap();

    // Convert bits into an INode struct
    let current_inode = unsafe { *(current_inode_data.as_mut_ptr() as *mut INode) };
    return current_inode;
}

fn write_inode(inode_number: u32, inode: &INode) {
    let block_index = inode_block!(inode_number);
    let mut current_data = read_sector(block_index) as [u8; BLOCK_SIZE as usize];
    let inode_index = inode_number % INODE_PER_BLOCK;
    let inode_start_address = (inode_index * INODE_SIZE) as usize;
    let inode_end_address = inode_start_address + INODE_SIZE as usize;

    // Take slice from data array
    let current_inode_data: &mut [u8; INODE_SIZE as usize] = current_data
        [inode_start_address..inode_end_address]
        .as_mut()
        .try_into()
        .unwrap();

    // Convert bits into an INode struct
    let current_inode = unsafe {
        (current_inode_data.as_mut_ptr() as *mut INode)
            .as_mut()
            .unwrap()
    };

    // Write new INode back to disk
    *current_inode = *inode;
    write_sector(block_index, current_data.to_vec());
}

fn mem_move(src: *const u8, dst: *mut u8, length: u32) {
    let mut count = 0_isize;
    while count != length as isize {
        unsafe { *dst.offset(count) = *src.offset(count) }
        count += 1;
    }
}

fn get_direct_block(
    inode: &mut INode,
    block_number: u32,
    next_free_data_block: &mut MutexGuard<'_, u32>,
) -> Option<u32> {
    // Requested block is indirect
    if block_number >= DIRECT_DATA_ADDRESS_SIZE as u32 {
        return None;
    }

    let current_block_value = inode.data[block_number as usize];

    // Block has not been allocated yet
    if current_block_value == 0 {
        inode.data[block_number as usize] = **next_free_data_block;
        **next_free_data_block += 1;
        return Some(inode.data[block_number as usize]);
    }

    // Block has been allocated
    Some(current_block_value)
}

fn get_indirect_block(
    inode: &mut INode,
    block_number: u32,
    next_free_data_block: &mut MutexGuard<'_, u32>,
) -> Option<u32> {
    // Requested block is direct
    if block_number < DIRECT_DATA_ADDRESS_SIZE as u32 {
        return None;
    }

    let mut current_indirect_value = inode.data[DIRECT_DATA_ADDRESS_SIZE];

    // Block has not been allocated yet
    if current_indirect_value == 0 {
        inode.data[DIRECT_DATA_ADDRESS_SIZE] = **next_free_data_block;
        **next_free_data_block += 1;
        current_indirect_value = inode.data[DIRECT_DATA_ADDRESS_SIZE];
    }

    let mut indirect_block_data = read_sector(current_indirect_value);
    let indirect_block_number = block_number as usize - DIRECT_DATA_ADDRESS_SIZE;
    let indirect_block_index = indirect_block_number * std::mem::size_of::<u32>();

    // Allocate new block in the indirect data block
    if indirect_block_data[indirect_block_index] == 0 {
        indirect_block_data[indirect_block_index..(indirect_block_index + 4)]
            .copy_from_slice((**next_free_data_block).to_le_bytes().as_ref());

        // Save indirect block
        write_sector(
            inode.data[DIRECT_DATA_ADDRESS_SIZE],
            indirect_block_data.to_vec(),
        );

        **next_free_data_block += 1;

        return Some(**next_free_data_block - 1);
    }

    let indirect_block = &indirect_block_data[indirect_block_index..(indirect_block_index + 4)];
    let indirect_block = u32::from_le_bytes(indirect_block.try_into().unwrap());
    Some(indirect_block)
}

fn get_inode_data_block(
    inode: &mut INode,
    block_number: u32,
    next_free_data_block: &mut MutexGuard<'_, u32>,
) -> u32 {
    let Some(block) = get_direct_block(inode, block_number, next_free_data_block) else {
        return get_indirect_block(inode, block_number, next_free_data_block).unwrap()
    };

    return block;
}

// This function writes the inode data into the data region. The idea is to break data into
// blocks of BLOCK_SIZE, writing it first to the direct data addresses (12) and then the indirect
// blocks (128).
fn append_inode(inode_number: u32, mut data: *const u8, mut length: u32) {
    let mut inode = read_inode(inode_number);
    let mut size = inode.size;
    let mut block_index = size / BLOCK_SIZE; // Block address we will be writing to
    let mut next_free_data_block = NEXT_FREE_DATA_BLOCK.lock().unwrap();

    while length > 0 {
        assert!(block_index < MAX_FILE_BLOCK);

        let data_block_number =
            get_inode_data_block(&mut inode, block_index, &mut next_free_data_block);

        // Write data block to allocated disk block and move to the next BLOCK_SIZE bytes
        let next_block_size = std::cmp::min(length, (block_index + 1) * BLOCK_SIZE - size);
        let mut buffer = read_sector(data_block_number);

        // Write buffer into the disk
        let buffer_offset = (size - block_index * BLOCK_SIZE) as isize;
        let buffer_pointer = unsafe { buffer.as_mut_ptr().offset(buffer_offset) };
        mem_move(data, buffer_pointer, next_block_size);
        write_sector(data_block_number, buffer.to_vec());

        size += next_block_size;
        length -= next_block_size;
        block_index = (size / BLOCK_SIZE) as u32;
        data = unsafe { data.offset(next_block_size as isize) };
    }

    // Write updated inode back to memory
    inode.size = size as u32;
    write_inode(inode_number, &inode);
}

fn allocate_inode(_type: INodeType, inode_counter: &mut u32) -> u32 {
    let inode_number = *inode_counter;
    *inode_counter += 1;

    let inode = INode {
        number_links: 1,
        size: 0,
        _type,
        ..Default::default()
    };

    write_inode(inode_number, &inode);
    inode_number
}

fn write_zero_sector(sector: u32) {
    let mut file_lock = FILE.lock().unwrap();
    let file = file_lock.by_ref();

    file.write_all_at(&ZERO_DATA, (BLOCK_SIZE * sector) as u64)
        .unwrap();
}

fn write_sector(sector: u32, data: Vec<u8>) {
    let mut file_lock = FILE.lock().unwrap();
    let file = file_lock.by_ref();

    file.write_all_at(&data, (BLOCK_SIZE * sector) as u64)
        .unwrap();
}

fn read_sector(sector: u32) -> [u8; BLOCK_SIZE as usize] {
    let mut data: [u8; BLOCK_SIZE as usize] = [0; BLOCK_SIZE as usize];
    let mut file_lock = FILE.lock().unwrap();
    let file = file_lock.by_ref();

    file.read_at(data.as_mut_slice(), (BLOCK_SIZE * sector) as u64)
        .expect("Failed to read at sector");

    data
}

fn write_super_block(offset: u32, data: &SuperBlock) {
    let serialized = serialize(&data).unwrap();
    let mut file_lock = FILE.lock().unwrap();
    let file = file_lock.by_ref();

    file.seek(SeekFrom::Start((BLOCK_SIZE * offset) as u64))
        .expect("Failed to seek file image");

    file.write_all(&serialized).unwrap();
}

fn open_file(path: &String) -> File {
    // Delete if file already exists
    let path = Path::new(path);
    if path.exists() {
        fs::remove_file(path).expect("Could not delete file");
    }

    return OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(path)
        .expect("Failed to open image file");
}

fn write_user_files(path: &String, inode_counter: &mut u32, root: u32) {
    let binaries = fs::read_dir(path).unwrap();

    for binary in binaries {
        let path = binary.as_ref().unwrap().path();
        let filename = binary.as_ref().unwrap().file_name();
        let inode = allocate_inode(INodeType::FILE, inode_counter);
        let dir_entry = &DirectoryEntry::new(inode, filename.to_str().unwrap())
            as *const DirectoryEntry as *const u8;

        // Append newly created file to root directory
        append_inode(root, dir_entry, size_of::<DirectoryEntry>() as u32);

        let mut file_data = OpenOptions::new()
            .read(true)
            .open(path.to_str().unwrap())
            .expect("Failed to open binary");

        // Read binary into vector buffer
        let mut buffer = Vec::new();
        std::io::Read::read_to_end(&mut file_data, &mut buffer).unwrap();

        append_inode(inode, buffer.as_ptr(), buffer.len() as u32);
    }
}

fn allocate_bitmap(number_used: u32) {
    let mut buffer = [0 as u8; BLOCK_SIZE as usize];
    println!("[MKFS] Used {} blocks", number_used);

    // Currently only a single free-block bitmap is supported. Remember each bitmap can
    // hold up to 4096 bits (512 bytes). Therefore, our File System supports up to 4096
    // blocks being used
    assert!(number_used < BLOCK_SIZE * 8);

    for block_number in 0..number_used {
        buffer[block_number as usize / 8] |= 1 << (block_number % 8)
    }

    write_sector(SUPER_BLOCK.bitmap_start_address, buffer.to_vec());
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 1 {
        eprintln!("mkfs requires at least 2 arguments, image of the file system and files...");
        return;
    }

    let mut inode_counter = 1;

    // Clear File System Image
    for sector in 0..FILE_SYSTEM_SIZE {
        write_zero_sector(sector);
    }

    // Sector at index 1 is always the superblock for the FS. It stores important information
    // regarding the File System, such as start addresses and sizes.
    let serialized = serialize(&SUPER_BLOCK).unwrap();
    write_sector(1, serialized);

    // Allocate root directory
    let root = allocate_inode(INodeType::DIRECTORY, &mut inode_counter);
    assert_eq!(root, 1); // Root must be the first inode

    // Create current (.) directory
    let root_directory = &DirectoryEntry::new(root, ".") as *const DirectoryEntry as *const u8;
    append_inode(root, root_directory, size_of::<DirectoryEntry>() as u32);

    // Create previous (..) directory
    let root_directory = &DirectoryEntry::new(root, "..") as *const DirectoryEntry as *const u8;
    append_inode(root, root_directory, size_of::<DirectoryEntry>() as u32);

    write_user_files(&args[2], &mut inode_counter, root);

    allocate_bitmap(*NEXT_FREE_DATA_BLOCK.lock().unwrap());
}

impl DirectoryEntry {
    pub fn new(inode: u32, name: &str) -> Self {
        let mut directory: DirectoryEntry = Default::default();
        directory.inode_number = inode;
        directory.name[..name.len()].copy_from_slice(name.as_bytes());
        directory
    }
}
