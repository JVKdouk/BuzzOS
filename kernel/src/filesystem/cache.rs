// TODO: Implement reference counter to cache blocks (perform MRU, perhaps Priority Queue instead of LL?)

use alloc::sync::Arc;

use crate::{
    scheduler::sleep::sleep, structures::heap_linked_list::HeapLinkedList,
    sync::spin_mutex::SpinMutex,
};

use super::ide::{request_ide, DiskBlock, DiskRequestStatus, BLOCK_SIZE};

const MAX_CACHE_BLOCKS: usize = 50; // Number of inodes to be kept in memory.

pub type CacheBlock = Arc<SpinMutex<DiskBlock>>;

static CACHE_BLOCK_LIST: SpinMutex<HeapLinkedList<CacheBlock>> =
    SpinMutex::new(HeapLinkedList::new());

impl HeapLinkedList<CacheBlock> {
    pub fn search(&self, device: u32, block_number: u32) -> Option<CacheBlock> {
        let mut current = &self.head;

        while current.is_some() {
            let data = &current.as_ref().unwrap().value.lock();

            if data.device == device && data.block_number == block_number {
                let clone = Arc::clone(&current.as_ref().unwrap().value);
                return Some(clone);
            }

            current = &current.as_ref().unwrap().next;
        }

        None
    }
}

pub fn store_new_block(device: u32, block_number: u32) -> CacheBlock {
    let mut cache = CACHE_BLOCK_LIST.lock();

    let block = Arc::new(SpinMutex::new({
        DiskBlock {
            device,
            block_number,
            dirty: false,
            status: DiskRequestStatus::AWAITING,
            data: [0; BLOCK_SIZE],
        }
    }));

    return Arc::clone(&cache.push(block).value);
}

pub fn write_disk_block(block: CacheBlock) -> CacheBlock {
    let address = block.lock().get_address();
    block.lock().dirty = true;

    request_ide(Arc::clone(&block));
    sleep(address);

    return block;
}

pub fn read_disk_block(device: u32, block_number: u32) -> CacheBlock {
    let block = get_cache_block(device, block_number).unwrap();

    // If block is already available in cache, return it
    if block.lock().status == DiskRequestStatus::READY {
        return block;
    }

    // Start IDE request
    let address = block.lock().get_address();
    request_ide(Arc::clone(&block));
    sleep(address);

    block
}

pub fn get_cache_block(device: u32, block_number: u32) -> Option<CacheBlock> {
    match CACHE_BLOCK_LIST.lock().search(device, block_number) {
        None => Some(store_new_block(device, block_number)),
        Some(block) => Some(block),
    }
}

/// Move the cache block to the head of the list (Most Recently Used) and reduce the reference
/// count to it. If the list is full, blocks with a reference count of 0 and not dirty are reused.
pub fn remove_cache_block(block: CacheBlock) {
    let cache = block.lock();
    let list = CACHE_BLOCK_LIST.lock();
}
