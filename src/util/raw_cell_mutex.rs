use std::cell::Cell;
use lock_api::{GuardNoSend, RawMutex};

pub struct RawCellMutex(Cell<bool>);

unsafe impl RawMutex for RawCellMutex {
    const INIT: Self = RawCellMutex(Cell::new(false));
    type GuardMarker = GuardNoSend;
    fn lock(&self) { assert!(!self.0.replace(true)); }
    fn try_lock(&self) -> bool { !self.0.replace(true) }
    unsafe fn unlock(&self) { assert!(self.0.replace(false)); }
}