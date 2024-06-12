//! Cached data for hashing accounts
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Default, Debug)]
pub(crate) struct CacheHashDataStats {
    pub(crate) cache_file_size: AtomicUsize,
    pub(crate) cache_file_count: AtomicUsize,
    pub(crate) total_entries: AtomicUsize,
    pub(crate) loaded_from_cache: AtomicUsize,
    pub(crate) entries_loaded_from_cache: AtomicUsize,
    pub(crate) save_us: AtomicU64,
    pub(crate) saved_to_cache: AtomicUsize,
    pub(crate) write_to_mmap_us: AtomicU64,
    pub(crate) create_save_us: AtomicU64,
    pub(crate) load_us: AtomicU64,
    pub(crate) read_us: AtomicU64,
    pub(crate) unused_cache_files: AtomicUsize,
    /// the number of hash data files that were found in the cache and reused
    pub(crate) hits: AtomicUsize,
    /// the number of hash data files that were not found in the cache
    pub(crate) misses: AtomicUsize,
}

impl CacheHashDataStats {
    pub(crate) fn report(&self) {
        datapoint_info!(
            "cache_hash_data_stats",
            (
                "cache_file_size",
                self.cache_file_size.load(Ordering::Relaxed),
                i64
            ),
            (
                "cache_file_count",
                self.cache_file_count.load(Ordering::Relaxed),
                i64
            ),
            (
                "total_entries",
                self.total_entries.load(Ordering::Relaxed),
                i64
            ),
            (
                "loaded_from_cache",
                self.loaded_from_cache.load(Ordering::Relaxed),
                i64
            ),
            (
                "saved_to_cache",
                self.saved_to_cache.load(Ordering::Relaxed),
                i64
            ),
            (
                "entries_loaded_from_cache",
                self.entries_loaded_from_cache.load(Ordering::Relaxed),
                i64
            ),
            ("save_us", self.save_us.load(Ordering::Relaxed), i64),
            (
                "write_to_mmap_us",
                self.write_to_mmap_us.load(Ordering::Relaxed),
                i64
            ),
            (
                "create_save_us",
                self.create_save_us.load(Ordering::Relaxed),
                i64
            ),
            ("load_us", self.load_us.load(Ordering::Relaxed), i64),
            ("read_us", self.read_us.load(Ordering::Relaxed), i64),
            (
                "unused_cache_files",
                self.unused_cache_files.load(Ordering::Relaxed),
                i64
            ),
            ("hits", self.hits.load(Ordering::Relaxed), i64),
            ("misses", self.misses.load(Ordering::Relaxed), i64),
        );
    }
}
