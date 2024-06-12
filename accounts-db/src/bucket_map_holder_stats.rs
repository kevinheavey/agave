use {
    crate::{
        accounts_index::{DiskIndexValue, IndexValue},
        bucket_map_holder::{Age, AtomicAge, BucketMapHolder},
    },
    solana_sdk::timing::AtomicInterval,
    std::{
        fmt::Debug,
        sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};

// stats logged every 10 s
const STATS_INTERVAL_MS: u64 = 10_000;

#[derive(Debug, Default)]
pub(crate) struct BucketMapHeldInMemStats {
    pub(crate) ref_count: AtomicU64,
    pub(crate) slot_list_len: AtomicU64,
    pub(crate) slot_list_cached: AtomicU64,
}

#[derive(Debug, Default)]
pub(crate) struct BucketMapHolderStats {
    pub(crate) held_in_mem: BucketMapHeldInMemStats,
    pub(crate) get_mem_us: AtomicU64,
    pub(crate) gets_from_mem: AtomicU64,
    pub(crate) get_missing_us: AtomicU64,
    pub(crate) gets_missing: AtomicU64,
    pub(crate) entry_mem_us: AtomicU64,
    pub(crate) entries_from_mem: AtomicU64,
    pub(crate) entry_missing_us: AtomicU64,
    pub(crate) entries_missing: AtomicU64,
    pub(crate) load_disk_found_count: AtomicU64,
    pub(crate) load_disk_found_us: AtomicU64,
    pub(crate) load_disk_missing_count: AtomicU64,
    pub(crate) load_disk_missing_us: AtomicU64,
    pub(crate) updates_in_mem: AtomicU64,
    pub(crate) items: AtomicU64,
    pub(crate) items_us: AtomicU64,
    pub(crate) failed_to_evict: AtomicU64,
    pub(crate) keys: AtomicU64,
    pub(crate) deletes: AtomicU64,
    pub(crate) buckets_scanned: AtomicU64,
    pub(crate) inserts: AtomicU64,
    count: AtomicUsize,
    pub(crate) bg_waiting_us: AtomicU64,
    pub(crate) bg_throttling_wait_us: AtomicU64,
    pub(crate) count_in_mem: AtomicUsize,
    pub(crate) per_bucket_count: Vec<AtomicUsize>,
    pub(crate) flush_entries_updated_on_disk: AtomicU64,
    pub(crate) flush_entries_evicted_from_mem: AtomicU64,
    pub(crate) active_threads: AtomicU64,
    pub(crate) get_range_us: AtomicU64,
    last_age: AtomicAge,
    last_ages_flushed: AtomicU64,
    pub(crate) flush_scan_us: AtomicU64,
    pub(crate) flush_update_us: AtomicU64,
    pub(crate) flush_evict_us: AtomicU64,
    pub(crate) flush_grow_us: AtomicU64,
    last_was_startup: AtomicBool,
    last_time: AtomicInterval,
    bins: u64,
    pub(crate) estimate_mem: AtomicU64,
    pub(crate) flush_should_evict_us: AtomicU64,
}

impl BucketMapHolderStats {
    pub(crate) fn new(bins: usize) -> BucketMapHolderStats {
        BucketMapHolderStats {
            bins: bins as u64,
            per_bucket_count: (0..bins).map(|_| AtomicUsize::default()).collect(),
            ..BucketMapHolderStats::default()
        }
    }

    pub(crate) fn inc_insert(&self) {
        self.inc_insert_count(1);
    }

    pub(crate) fn inc_insert_count(&self, count: u64) {
        self.inserts.fetch_add(count, Ordering::Relaxed);
        self.count.fetch_add(count as usize, Ordering::Relaxed);
    }

    pub(crate) fn inc_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
        self.count.fetch_sub(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_mem_count(&self, bin: usize) {
        self.add_mem_count(bin, 1);
    }

    pub(crate) fn dec_mem_count(&self, bin: usize) {
        self.sub_mem_count(bin, 1);
    }

    pub(crate) fn add_mem_count(&self, bin: usize, count: usize) {
        let per_bucket = self.per_bucket_count.get(bin);
        self.count_in_mem.fetch_add(count, Ordering::Relaxed);
        per_bucket.map(|stat| stat.fetch_add(count, Ordering::Relaxed));
    }

    pub(crate) fn sub_mem_count(&self, bin: usize, count: usize) {
        let per_bucket = self.per_bucket_count.get(bin);
        self.count_in_mem.fetch_sub(count, Ordering::Relaxed);
        per_bucket.map(|stat| stat.fetch_sub(count, Ordering::Relaxed));
    }

    fn ms_per_age<T: IndexValue, U: DiskIndexValue + From<T> + Into<T>>(
        &self,
        storage: &BucketMapHolder<T, U>,
        elapsed_ms: u64,
    ) -> u64 {
        let age_now = storage.current_age();
        let ages_flushed = storage.count_buckets_flushed() as u64;
        let last_age = self.last_age.swap(age_now, Ordering::Relaxed) as u64;
        let last_ages_flushed = self.last_ages_flushed.swap(ages_flushed, Ordering::Relaxed);
        let mut age_now = age_now as u64;
        if last_age > age_now {
            // age wrapped
            age_now += Age::MAX as u64 + 1;
        }
        let age_delta = age_now.saturating_sub(last_age);
        if age_delta > 0 {
            return elapsed_ms / age_delta;
        } else {
            // did not advance an age, but probably did partial work, so report that
            let bin_delta = ages_flushed.saturating_sub(last_ages_flushed);
            if bin_delta > 0 {
                return elapsed_ms * self.bins / bin_delta;
            }
        }
        0 // avoid crazy numbers
    }

    pub(crate) fn remaining_until_next_interval(&self) -> u64 {
        self.last_time
            .remaining_until_next_interval(STATS_INTERVAL_MS)
    }

    /// return min, max, sum, median of data
    fn get_stats(mut data: Vec<usize>) -> (usize, usize, usize, usize) {
        if data.is_empty() {
            (0, 0, 0, 0)
        } else {
            data.sort_unstable();
            (
                *data.first().unwrap(),
                *data.last().unwrap(),
                data.iter().sum(),
                data[data.len() / 2],
            )
        }
    }

    fn calc_percent(ms: u64, elapsed_ms: u64) -> f32 {
        if elapsed_ms == 0 {
            0.0
        } else {
            (ms as f32 / elapsed_ms as f32) * 100.0
        }
    }

    pub(crate) fn total_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub(crate) fn count_in_bucket(&self, bucket: usize) -> usize {
        if bucket < self.per_bucket_count.len() {
            self.per_bucket_count[bucket].load(Ordering::Relaxed)
        } else {
            0
        }
    }

    /// This is an estimate of the # of items in mem that are awaiting flushing to disk.
    /// returns (# items in mem) - (# items we intend to hold in mem for performance heuristics)
    /// The result is also an estimate because 'held_in_mem' is based on a stat that is swapped out when stats are reported.
    pub(crate) fn get_remaining_items_to_flush_estimate(&self) -> usize {
        let in_mem = self.count_in_mem.load(Ordering::Relaxed) as u64;
        let held_in_mem = self.held_in_mem.slot_list_cached.load(Ordering::Relaxed)
            + self.held_in_mem.slot_list_len.load(Ordering::Relaxed)
            + self.held_in_mem.ref_count.load(Ordering::Relaxed);
        in_mem.saturating_sub(held_in_mem) as usize
    }

    pub(crate) fn report_stats<T: IndexValue, U: DiskIndexValue + From<T> + Into<T>>(
        &self,
        storage: &BucketMapHolder<T, U>,
    ) {
        let elapsed_ms = self.last_time.elapsed_ms();
        if elapsed_ms < STATS_INTERVAL_MS {
            return;
        }

        if !self.last_time.should_update(STATS_INTERVAL_MS) {
            return;
        }

        let ms_per_age = self.ms_per_age(storage, elapsed_ms);

        let in_mem_per_bucket_counts = self
            .per_bucket_count
            .iter()
            .map(|count| count.load(Ordering::Relaxed))
            .collect::<Vec<_>>();
        let disk = storage.disk.as_ref();
        let disk_per_bucket_counts = disk
            .map(|disk| {
                (0..self.bins)
                    .map(|i| disk.get_bucket_from_index(i as usize).bucket_len() as usize)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let in_mem_stats = Self::get_stats(in_mem_per_bucket_counts);
        let disk_stats = Self::get_stats(disk_per_bucket_counts);

        const US_PER_MS: u64 = 1_000;

        // all metrics during startup are written to a different data point
        let startup = storage.get_startup();
        let was_startup = self.last_was_startup.swap(startup, Ordering::Relaxed);

        // sum of elapsed time in each thread
        let mut thread_time_elapsed_ms = elapsed_ms * storage.threads as u64;
        if disk.is_some() {
            if was_startup {
                // these stats only apply at startup
                datapoint_info!(
                    "accounts_index_startup",
                    (
                        "entries_created",
                        disk.map(|disk| disk
                            .stats
                            .index
                            .startup
                            .entries_created
                            .swap(0, Ordering::Relaxed))
                            .unwrap_or_default(),
                        i64
                    ),
                    (
                        "entries_reused",
                        disk.map(|disk| disk
                            .stats
                            .index
                            .startup
                            .entries_reused
                            .swap(0, Ordering::Relaxed))
                            .unwrap_or_default(),
                        i64
                    ),
                );
            }
            datapoint_info!(
                if startup || was_startup {
                    thread_time_elapsed_ms *= 2; // more threads are allocated during startup
                    "accounts_index_startup"
                } else {
                    "accounts_index"
                },
                (
                    "estimate_mem_bytes",
                    self.estimate_mem.load(Ordering::Relaxed),
                    i64
                ),
                (
                    "flush_should_evict_us",
                    self.flush_should_evict_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "count_in_mem",
                    self.count_in_mem.load(Ordering::Relaxed),
                    i64
                ),
                ("count", self.total_count(), i64),
                (
                    "bg_waiting_percent",
                    Self::calc_percent(
                        self.bg_waiting_us.swap(0, Ordering::Relaxed) / US_PER_MS,
                        thread_time_elapsed_ms
                    ),
                    f64
                ),
                (
                    "bg_throttling_wait_percent",
                    Self::calc_percent(
                        self.bg_throttling_wait_us.swap(0, Ordering::Relaxed) / US_PER_MS,
                        thread_time_elapsed_ms
                    ),
                    f64
                ),
                (
                    "slot_list_len",
                    self.held_in_mem.slot_list_len.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "ref_count",
                    self.held_in_mem.ref_count.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "slot_list_cached",
                    self.held_in_mem.slot_list_cached.swap(0, Ordering::Relaxed),
                    i64
                ),
                ("min_in_bin_mem", in_mem_stats.0, i64),
                ("max_in_bin_mem", in_mem_stats.1, i64),
                ("count_from_bins_mem", in_mem_stats.2, i64),
                ("median_from_bins_mem", in_mem_stats.3, i64),
                ("min_in_bin_disk", disk_stats.0, i64),
                ("max_in_bin_disk", disk_stats.1, i64),
                ("count_from_bins_disk", disk_stats.2, i64),
                ("median_from_bins_disk", disk_stats.3, i64),
                (
                    "gets_from_mem",
                    self.gets_from_mem.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "get_mem_us",
                    self.get_mem_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "gets_missing",
                    self.gets_missing.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "get_missing_us",
                    self.get_missing_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entries_from_mem",
                    self.entries_from_mem.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entry_mem_us",
                    self.entry_mem_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "load_disk_found_count",
                    self.load_disk_found_count.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "load_disk_found_us",
                    self.load_disk_found_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "load_disk_missing_count",
                    self.load_disk_missing_count.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "load_disk_missing_us",
                    self.load_disk_missing_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entries_missing",
                    self.entries_missing.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entry_missing_us",
                    self.entry_missing_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "failed_to_evict",
                    self.failed_to_evict.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "updates_in_mem",
                    self.updates_in_mem.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "get_range_us",
                    self.get_range_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                ("inserts", self.inserts.swap(0, Ordering::Relaxed), i64),
                ("deletes", self.deletes.swap(0, Ordering::Relaxed), i64),
                (
                    "active_threads",
                    self.active_threads.load(Ordering::Relaxed),
                    i64
                ),
                ("items", self.items.swap(0, Ordering::Relaxed), i64),
                ("keys", self.keys.swap(0, Ordering::Relaxed), i64),
                ("ms_per_age", ms_per_age, i64),
                (
                    "buckets_scanned",
                    self.buckets_scanned.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "flush_scan_us",
                    self.flush_scan_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "flush_update_us",
                    self.flush_update_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "flush_grow_us",
                    self.flush_grow_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "flush_evict_us",
                    self.flush_evict_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "disk_index_resizes",
                    disk.map(|disk| disk.stats.index.resizes.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_failed_resizes",
                    disk.map(|disk| disk.stats.index.failed_resizes.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_max_size",
                    disk.map(|disk| { disk.stats.index.max_size.swap(0, Ordering::Relaxed) })
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_new_file_us",
                    disk.map(|disk| disk.stats.index.new_file_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_resize_us",
                    disk.map(|disk| disk.stats.index.resize_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_flush_file_us",
                    disk.map(|disk| disk.stats.index.flush_file_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_index_file_size",
                    disk.map(|disk| disk.stats.index.total_file_size.load(Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_data_file_size",
                    disk.map(|disk| disk.stats.data.total_file_size.load(Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_data_file_count",
                    disk.map(|disk| disk.stats.data.file_count.load(Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_find_index_entry_mut_us",
                    disk.map(|disk| disk
                        .stats
                        .index
                        .find_index_entry_mut_us
                        .swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_index_flush_mmap_us",
                    disk.map(|disk| disk.stats.index.mmap_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_data_resizes",
                    disk.map(|disk| disk.stats.data.resizes.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_data_max_size",
                    disk.map(|disk| { disk.stats.data.max_size.swap(0, Ordering::Relaxed) })
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_data_new_file_us",
                    disk.map(|disk| disk.stats.data.new_file_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_data_resize_us",
                    disk.map(|disk| disk.stats.data.resize_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_data_flush_file_us",
                    disk.map(|disk| disk.stats.data.flush_file_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "disk_data_flush_mmap_us",
                    disk.map(|disk| disk.stats.data.mmap_us.swap(0, Ordering::Relaxed))
                        .unwrap_or_default(),
                    i64
                ),
                (
                    "flush_entries_updated_on_disk",
                    self.flush_entries_updated_on_disk
                        .swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "flush_entries_evicted_from_mem",
                    self.flush_entries_evicted_from_mem
                        .swap(0, Ordering::Relaxed),
                    i64
                ),
            );
        } else {
            datapoint_info!(
                if startup || was_startup {
                    thread_time_elapsed_ms *= 2; // more threads are allocated during startup
                    "accounts_index_startup"
                } else {
                    "accounts_index"
                },
                (
                    "count_in_mem",
                    self.count_in_mem.load(Ordering::Relaxed),
                    i64
                ),
                ("count", self.total_count(), i64),
                (
                    "bg_waiting_percent",
                    Self::calc_percent(
                        self.bg_waiting_us.swap(0, Ordering::Relaxed) / US_PER_MS,
                        thread_time_elapsed_ms
                    ),
                    f64
                ),
                (
                    "bg_throttling_wait_percent",
                    Self::calc_percent(
                        self.bg_throttling_wait_us.swap(0, Ordering::Relaxed) / US_PER_MS,
                        thread_time_elapsed_ms
                    ),
                    f64
                ),
                ("min_in_bin_mem", in_mem_stats.0, i64),
                ("max_in_bin_mem", in_mem_stats.1, i64),
                ("count_from_bins_mem", in_mem_stats.2, i64),
                ("median_from_bins_mem", in_mem_stats.3, i64),
                (
                    "gets_from_mem",
                    self.gets_from_mem.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "get_mem_us",
                    self.get_mem_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "gets_missing",
                    self.gets_missing.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "get_missing_us",
                    self.get_missing_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entries_from_mem",
                    self.entries_from_mem.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entry_mem_us",
                    self.entry_mem_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entries_missing",
                    self.entries_missing.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "entry_missing_us",
                    self.entry_missing_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "updates_in_mem",
                    self.updates_in_mem.swap(0, Ordering::Relaxed),
                    i64
                ),
                (
                    "get_range_us",
                    self.get_range_us.swap(0, Ordering::Relaxed),
                    i64
                ),
                ("inserts", self.inserts.swap(0, Ordering::Relaxed), i64),
                ("deletes", self.deletes.swap(0, Ordering::Relaxed), i64),
                (
                    "active_threads",
                    self.active_threads.load(Ordering::Relaxed),
                    i64
                ),
                ("items", self.items.swap(0, Ordering::Relaxed), i64),
                ("items_us", self.items_us.swap(0, Ordering::Relaxed), i64),
                ("keys", self.keys.swap(0, Ordering::Relaxed), i64),
            );
        }
    }
}
