use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use cache_padded::CachePadded;
use crate::concurrent_fixed::CapacityError;

pub trait BuildLoadFactor: Send + Sync {
    type LoadFactor: LoadFactor;
    fn build_load_factor(&self, cap: usize) -> Self::LoadFactor;
}

pub trait LoadFactor: Send + Sync {
    fn try_insert(&self, index: usize) -> Result<(), CapacityError<()>>;
    fn is_full(&self) -> bool;
}

const MAX_LOAD_FACTOR: f64 = 0.75;
const MIN_BUDGET: usize = 128;
const MAX_DOWNSAMPLE: usize = 16;

#[derive(Default)]
pub struct BuildSampledLoadFactor;

pub struct SampledLoadFactor {
    load_mask: usize,
    load_budget: CachePadded<AtomicUsize>,
}

#[derive(Default)]
pub struct BuildNoLoadFactor;

pub struct NoLoadFactor;

impl BuildLoadFactor for BuildSampledLoadFactor {
    type LoadFactor = SampledLoadFactor;
    fn build_load_factor(&self, cap: usize) -> SampledLoadFactor {
        let mut load_downsample = 1;
        let mut load_budget = ((cap as f64) * MAX_LOAD_FACTOR) as usize;
        while load_budget > MIN_BUDGET && load_downsample < MAX_DOWNSAMPLE {
            load_downsample *= 2;
            load_budget /= 2;
        }
        SampledLoadFactor {
            load_mask: load_downsample - 1,
            load_budget: CachePadded::new(AtomicUsize::new(load_budget)),
        }
    }
}

impl LoadFactor for SampledLoadFactor {
    fn try_insert(&self, index: usize) -> Result<(), CapacityError<()>> {
        if index & self.load_mask == 0 {
            if self.load_budget.fetch_update(
                Relaxed, Relaxed, |x| x.checked_sub(1))
                .is_err() {
                return Err(CapacityError(()));
            }
        } else {
            if self.load_budget.load(Relaxed) == 0 {
                return Err(CapacityError(()));
            }
        }
        Ok(())
    }

    fn is_full(&self) -> bool {
        self.load_budget.load(Relaxed) == 0
    }
}

impl BuildLoadFactor for BuildNoLoadFactor {
    type LoadFactor = NoLoadFactor;
    fn build_load_factor(&self, cap: usize) -> Self::LoadFactor {
        NoLoadFactor
    }
}

impl LoadFactor for NoLoadFactor {
    fn try_insert(&self, index: usize) -> Result<(), CapacityError<()>> { Ok(()) }
    fn is_full(&self) -> bool { false }
}

// pub trait LoadFactor2{
//
// }
//
// pub struct SampledLoadFactor2 {
//     sample_mask: usize,
//     load: CachePadded<AtomicUsize>,
// }