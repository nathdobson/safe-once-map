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

//!
//! For single-threaded code:
//! ```
//! # use safe_once::unsync::OnceCell;
//! # use safe_once_map::unsync::OnceCellMap;
//! let map = OnceCellMap::<String, i32>::new();
//! let cell: &OnceCell<i32> = &map["a"];
//! assert_eq!(4, *map["a"].get_or_init(|| 4));
//! assert_eq!(4, *map["a"].get_or_init(|| 8));
//! ```
//!
//! For multi-threaded code:
//! ```
//! # use safe_once::sync::OnceLock;
//! # use safe_once_map::sync::OnceLockMap;
//! let map = OnceLockMap::<String, i32>::new();
//! let cell: &OnceLock<i32> = &map["a"];
//! assert_eq!(4, *map["a"].get_or_init(|| 4));
//! assert_eq!(4, *map["a"].get_or_init(|| 8));
//! ```
//!
#[cfg(feature = "unsync")]
pub mod raw_mutex_cell;
#[cfg(feature = "unsync")]
pub mod unsync;
#[cfg(feature = "sync")]
pub mod sync;
pub mod raw_map;
pub mod stable_map;
