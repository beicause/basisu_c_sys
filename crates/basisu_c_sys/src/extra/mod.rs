#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
mod encoder;
mod transcoder;

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
pub use encoder::*;
pub use transcoder::*;

use crate::transcoder as trans_sys;
use alloc::vec::Vec;
use core::num::NonZero;
use core::ops::{Bound, Range, RangeBounds};

pub mod types;

#[derive(Debug, thiserror::Error)]
pub(crate) enum BuHeapAccessError {
    #[error("Invalid {range:?}, end < start")]
    InvalidRange { range: Range<u64> },
    #[error("Read/Write out of bounds, the capacity is {capacity} but the end is {end}")]
    OutOfBounds { end: u64, capacity: u64 },
}

/// Safe wrapper of basisu memory allocation ([`trans_sys::bt_alloc`] and [`trans_sys::bt_free`]).
///
// Note: This is used for both transcoder and encoder and assumes their `alloc` and `free` are the same. If not (though it's unlikely), this should be changed.
pub(crate) struct BuHeap {
    ptr: NonZero<u64>,
    capacity: NonZero<u64>,
}

impl BuHeap {
    pub fn new_uninit(capacity: NonZero<u64>) -> Self {
        unsafe {
            let ptr = trans_sys::bt_alloc(capacity.into());
            Self {
                ptr: NonZero::new(ptr).expect("basisu alloc failed"),
                capacity,
            }
        }
    }
    pub fn new(data: &[u8]) -> Option<Self> {
        let capacity = NonZero::new(data.len() as u64)?;
        let mut bt = Self::new_uninit(capacity);
        bt.try_write(0, data).unwrap();
        Some(bt)
    }
    #[inline]
    pub fn ptr(&self) -> NonZero<u64> {
        self.ptr
    }
    #[inline]
    pub fn capacity(&self) -> NonZero<u64> {
        self.capacity
    }
    pub fn try_read<S: RangeBounds<u64>>(&self, range: S) -> Result<Vec<u8>, BuHeapAccessError> {
        let capacity = u64::from(self.capacity());
        let mut r = 0..capacity;
        match range.start_bound() {
            Bound::Included(&start_idx) => {
                r.start = start_idx;
            }
            Bound::Excluded(&start) => {
                r.start = start + 1;
            }
            Bound::Unbounded => {}
        }
        match range.end_bound() {
            Bound::Included(&end_idx) => {
                r.end = end_idx + 1;
            }
            Bound::Excluded(&end) => {
                r.end = end;
            }
            Bound::Unbounded => {}
        }
        if r.end < r.start {
            return Err(BuHeapAccessError::InvalidRange { range: r });
        }
        if r.end > capacity {
            return Err(BuHeapAccessError::OutOfBounds {
                end: r.end,
                capacity,
            });
        }

        if r.end == r.start {
            return Ok(Vec::new());
        }

        Ok(unsafe {
            crate::copy_basisu_memory_to_host(u64::from(self.ptr()) + r.start, r.end - r.start)
        })
    }
    pub fn try_write(&mut self, offset: u64, data: &[u8]) -> Result<(), BuHeapAccessError> {
        let capacity = u64::from(self.capacity());
        let end = offset + data.len() as u64;
        if end > capacity {
            return Err(BuHeapAccessError::OutOfBounds { end, capacity });
        }

        unsafe { crate::copy_host_memory_to_basisu(data, u64::from(self.ptr()) + offset) };
        Ok(())
    }
}
impl Drop for BuHeap {
    fn drop(&mut self) {
        unsafe {
            trans_sys::bt_free(u64::from(self.ptr()));
        }
    }
}
