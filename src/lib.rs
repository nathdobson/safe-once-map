#![feature(hash_raw_entry)]
#![feature(default_free_fn)]
#![feature(trait_alias)]
#![feature(test)]
#![deny(unused_must_use)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![feature(type_alias_impl_trait)]
#![feature(box_syntax)]
#![feature(sort_floats)]
#![feature(build_hasher_simple_hash_one)]
#![feature(map_try_insert)]
#![allow(unused_assignments)]

extern crate test;

#[cfg(feature = "unsync")]
mod raw_mutex_cell;
#[cfg(feature = "unsync")]
mod unsync;
#[cfg(feature = "sync")]
mod sync;
mod unbounded;
mod unstable_map;
mod stable_map;

#[cfg(feature = "unsync")]
pub use unsync::*;

#[cfg(feature = "sync")]
pub use sync::*;
