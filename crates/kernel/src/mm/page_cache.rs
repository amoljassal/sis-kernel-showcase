/// Page Cache / Buffer Cache for Block I/O
///
/// Implements an LRU cache for block device sectors to improve I/O performance.
/// Each cached block (BufferHead) tracks dirty state and supports write-back.

use crate::lib::error::{Result, Errno};
use crate::block::BlockDevice;
use alloc::sync::Arc;
use alloc::vec::{self, Vec};
use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Cache key: (device major/minor, sector number)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Device major number
    pub major: u32,
    /// Device minor number
    pub minor: u32,
    /// Sector number (LBA)
    pub sector: u64,
}

impl CacheKey {
    pub fn new(device: &BlockDevice, sector: u64) -> Self {
        Self {
            major: device.major,
            minor: device.minor,
            sector,
        }
    }
}

/// Buffer head - represents a cached block
pub struct BufferHead {
    /// Cached sector data (typically 512 bytes)
    data: Mutex<Vec<u8>>,
    /// Device reference
    device: Arc<BlockDevice>,
    /// Sector number
    sector: u64,
    /// Dirty flag (needs write-back)
    dirty: AtomicBool,
    /// Reference count
    refcount: AtomicU64,
}

impl BufferHead {
    /// Create a new buffer head
    fn new(device: Arc<BlockDevice>, sector: u64, data: Vec<u8>) -> Self {
        Self {
            data: Mutex::new(data),
            device,
            sector,
            dirty: AtomicBool::new(false),
            refcount: AtomicU64::new(1),
        }
    }

    /// Mark buffer as dirty
    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    /// Check if buffer is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }

    /// Get immutable reference to data
    pub fn data(&self) -> spin::MutexGuard<Vec<u8>> {
        self.data.lock()
    }

    /// Get mutable reference to data (marks dirty)
    pub fn data_mut(&self) -> spin::MutexGuard<Vec<u8>> {
        self.mark_dirty();
        self.data.lock()
    }

    /// Write back dirty data to device
    pub fn sync(&self) -> Result<()> {
        if !self.is_dirty() {
            return Ok(());
        }

        let data = self.data.lock();
        self.device.write_sectors(self.sector, &data)?;
        self.dirty.store(false, Ordering::Release);

        Ok(())
    }

    /// Increment reference count
    fn get(&self) {
        self.refcount.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement reference count
    fn put(&self) -> u64 {
        self.refcount.fetch_sub(1, Ordering::Relaxed) - 1
    }

    /// Get current reference count
    fn refs(&self) -> u64 {
        self.refcount.load(Ordering::Relaxed)
    }
}

/// LRU cache entry
struct CacheEntry {
    key: CacheKey,
    buffer: Arc<BufferHead>,
}

/// Page cache with LRU eviction
pub struct PageCache {
    /// LRU queue (most recently used at back)
    lru: Mutex<VecDeque<CacheEntry>>,
    /// Maximum number of cached blocks
    max_blocks: usize,
    /// Current number of cached blocks
    cached_blocks: AtomicU64,
    /// Cache hit count
    hits: AtomicU64,
    /// Cache miss count
    misses: AtomicU64,
}

impl PageCache {
    /// Create a new page cache
    pub fn new(max_blocks: usize) -> Self {
        Self {
            lru: Mutex::new(VecDeque::with_capacity(max_blocks)),
            max_blocks,
            cached_blocks: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Get a buffer from cache or read from device
    pub fn get_buffer(&self, device: Arc<BlockDevice>, sector: u64) -> Result<Arc<BufferHead>> {
        let key = CacheKey::new(&device, sector);

        // Try to find in cache
        {
            let mut lru = self.lru.lock();
            if let Some(pos) = lru.iter().position(|entry| entry.key == key) {
                // Cache hit - move to back (most recently used)
                let entry = lru.remove(pos).unwrap();
                entry.buffer.get();
                lru.push_back(CacheEntry {
                    key: entry.key,
                    buffer: entry.buffer.clone(),
                });
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Ok(entry.buffer);
            }
        }

        // Cache miss - read from device
        self.misses.fetch_add(1, Ordering::Relaxed);

        let mut data = vec![0u8; device.sector_size];
        device.read_sectors(sector, &mut data)?;

        let buffer = Arc::new(BufferHead::new(device, sector, data));

        // Add to cache
        {
            let mut lru = self.lru.lock();

            // Evict if cache is full
            if lru.len() >= self.max_blocks {
                self.evict_lru(&mut lru)?;
            }

            lru.push_back(CacheEntry {
                key,
                buffer: buffer.clone(),
            });

            self.cached_blocks.store(lru.len() as u64, Ordering::Relaxed);
        }

        Ok(buffer)
    }

    /// Release a buffer (decrement reference count)
    pub fn put_buffer(&self, buffer: Arc<BufferHead>) {
        buffer.put();
        // Note: Buffer stays in cache even when refcount reaches 0
        // It will be evicted by LRU policy if needed
    }

    /// Evict least recently used buffer
    fn evict_lru(&self, lru: &mut VecDeque<CacheEntry>) -> Result<()> {
        // Find first buffer with refcount == 0
        for i in 0..lru.len() {
            if lru[i].buffer.refs() == 0 {
                let entry = lru.remove(i).unwrap();

                // Write back if dirty
                if entry.buffer.is_dirty() {
                    entry.buffer.sync()?;
                }

                return Ok(());
            }
        }

        // All buffers are in use - return ENOMEM
        crate::warn!("page_cache: cannot evict, all buffers in use");
        Err(Errno::ENOMEM)
    }

    /// Flush all dirty buffers to disk
    pub fn sync_all(&self) -> Result<()> {
        let lru = self.lru.lock();

        for entry in lru.iter() {
            if entry.buffer.is_dirty() {
                entry.buffer.sync()?;
            }
        }

        Ok(())
    }

    /// Flush dirty buffers for a specific device
    pub fn sync_device(&self, major: u32, minor: u32) -> Result<()> {
        let lru = self.lru.lock();

        for entry in lru.iter() {
            if entry.key.major == major && entry.key.minor == minor {
                if entry.buffer.is_dirty() {
                    entry.buffer.sync()?;
                }
            }
        }

        Ok(())
    }

    /// Invalidate all buffers for a device (called on unmount)
    pub fn invalidate_device(&self, major: u32, minor: u32) -> Result<()> {
        // First, sync any dirty buffers
        self.sync_device(major, minor)?;

        // Remove all buffers for this device
        let mut lru = self.lru.lock();
        lru.retain(|entry| {
            !(entry.key.major == major && entry.key.minor == minor && entry.buffer.refs() == 0)
        });

        self.cached_blocks.store(lru.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            cached_blocks: self.cached_blocks.load(Ordering::Relaxed),
            max_blocks: self.max_blocks as u64,
            hits,
            misses,
            hit_rate,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cached_blocks: u64,
    pub max_blocks: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

impl core::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PageCache: {}/{} blocks, {} hits, {} misses, {:.1}% hit rate",
               self.cached_blocks, self.max_blocks,
               self.hits, self.misses, self.hit_rate)
    }
}

/// Global page cache instance
static PAGE_CACHE: Mutex<Option<PageCache>> = Mutex::new(None);

/// Initialize the page cache
pub fn init_page_cache(max_blocks: usize) {
    let cache = PageCache::new(max_blocks);
    *PAGE_CACHE.lock() = Some(cache);
    crate::info!("page_cache: initialized with {} blocks", max_blocks);
}

/// Get a buffer from the global page cache
pub fn get_buffer(device: Arc<BlockDevice>, sector: u64) -> Result<Arc<BufferHead>> {
    let cache = PAGE_CACHE.lock();
    let cache = cache.as_ref().ok_or(Errno::EINVAL)?;
    cache.get_buffer(device, sector)
}

/// Release a buffer to the global page cache
pub fn put_buffer(buffer: Arc<BufferHead>) {
    if let Some(cache) = PAGE_CACHE.lock().as_ref() {
        cache.put_buffer(buffer);
    }
}

/// Sync all dirty buffers
pub fn sync_all() -> Result<()> {
    let cache = PAGE_CACHE.lock();
    let cache = cache.as_ref().ok_or(Errno::EINVAL)?;
    cache.sync_all()
}

/// Sync buffers for a specific device
pub fn sync_device(device: &BlockDevice) -> Result<()> {
    let cache = PAGE_CACHE.lock();
    let cache = cache.as_ref().ok_or(Errno::EINVAL)?;
    cache.sync_device(device.major, device.minor)
}

/// Invalidate buffers for a device
pub fn invalidate_device(device: &BlockDevice) -> Result<()> {
    let cache = PAGE_CACHE.lock();
    let cache = cache.as_ref().ok_or(Errno::EINVAL)?;
    cache.invalidate_device(device.major, device.minor)
}

/// Get cache statistics
pub fn cache_stats() -> Option<CacheStats> {
    PAGE_CACHE.lock().as_ref().map(|c| c.stats())
}
