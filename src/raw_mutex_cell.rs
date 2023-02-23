use std::cell::Cell;
use lock_api::{GuardNoSend, RawMutex};

pub struct RawMutexCell(Cell<bool>);

unsafe impl RawMutex for RawMutexCell {
    const INIT: Self = RawMutexCell(Cell::new(false));
    type GuardMarker = GuardNoSend;
    #[track_caller]
    fn lock(&self) {
        assert!(!self.0.replace(true));
    }
    #[track_caller]
    fn try_lock(&self) -> bool {
        !self.0.replace(true)
    }
    #[track_caller]
    unsafe fn unlock(&self) {
        assert!(self.0.replace(false));
    }
}