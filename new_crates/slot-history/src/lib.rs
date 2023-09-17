//! A type to hold data for the [`SlotHistory` sysvar][sv].
//!
//! [sv]: https://docs.solana.com/developing/runtime-facilities/sysvars#slothistory
//!
//! The sysvar ID is declared in [`sysvar::slot_history`].
//!
//! [`sysvar::slot_history`]: crate::sysvar::slot_history

#![allow(clippy::arithmetic_side_effects)]
pub use solana_clock::Slot;
use {
    bv::{BitVec, BitsMut},
    serde::{Deserialize, Serialize},
};

/// A bitvector indicating which slots are present in the past epoch.
#[repr(C)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotHistory {
    pub bits: BitVec<u64>,
    pub next_slot: Slot,
}

impl Default for SlotHistory {
    fn default() -> Self {
        let mut bits = BitVec::new_fill(false, MAX_ENTRIES);
        bits.set(0, true);
        Self { bits, next_slot: 1 }
    }
}

impl std::fmt::Debug for SlotHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SlotHistory {{ slot: {} bits:", self.next_slot)?;
        for i in 0..MAX_ENTRIES {
            if self.bits.get(i) {
                write!(f, "1")?;
            } else {
                write!(f, "0")?;
            }
        }
        Ok(())
    }
}

pub const MAX_ENTRIES: u64 = 1024 * 1024; // 1 million slots is about 5 days

#[derive(PartialEq, Eq, Debug)]
pub enum Check {
    Future,
    TooOld,
    Found,
    NotFound,
}

impl SlotHistory {
    pub fn add(&mut self, slot: Slot) {
        if slot > self.next_slot && slot - self.next_slot >= MAX_ENTRIES {
            // Wrapped past current history,
            // clear entire bitvec.
            let full_blocks = (MAX_ENTRIES as usize) / 64;
            for i in 0..full_blocks {
                self.bits.set_block(i, 0);
            }
        } else {
            for skipped in self.next_slot..slot {
                self.bits.set(skipped % MAX_ENTRIES, false);
            }
        }
        self.bits.set(slot % MAX_ENTRIES, true);
        self.next_slot = slot + 1;
    }

    pub fn check(&self, slot: Slot) -> Check {
        if slot > self.newest() {
            Check::Future
        } else if slot < self.oldest() {
            Check::TooOld
        } else if self.bits.get(slot % MAX_ENTRIES) {
            Check::Found
        } else {
            Check::NotFound
        }
    }

    pub fn oldest(&self) -> Slot {
        self.next_slot.saturating_sub(MAX_ENTRIES)
    }

    pub fn newest(&self) -> Slot {
        self.next_slot - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_older_slot() {
        let mut slot_history = SlotHistory::default();
        slot_history.add(10);
        slot_history.add(5);
        assert_eq!(slot_history.check(0), Check::Found);
        assert_eq!(slot_history.check(5), Check::Found);
        // If we go backwards we reset?
        assert_eq!(slot_history.check(10), Check::Future);
        assert_eq!(slot_history.check(6), Check::Future);
        assert_eq!(slot_history.check(11), Check::Future);
    }

    #[test]
    fn test_oldest() {
        let mut slot_history = SlotHistory::default();
        assert_eq!(slot_history.oldest(), 0);
        slot_history.add(10);
        assert_eq!(slot_history.oldest(), 0);
        slot_history.add(MAX_ENTRIES - 1);
        assert_eq!(slot_history.oldest(), 0);
        slot_history.add(MAX_ENTRIES);
        assert_eq!(slot_history.oldest(), 1);
    }
}
