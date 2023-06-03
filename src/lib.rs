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
#![feature(build_hasher_simple_hash_one)]
#![feature(map_try_insert)]
#![feature(type_name_of_val)]
#![feature(unboxed_closures)]
#![allow(unused_assignments)]
#![feature(fn_traits)]
#![feature(tuple_trait)]
#![allow(unused_mut)]

//! Utilities for caching and memoization.
//!
//! ## Closure
//! A closure can be memoized with [`LazyMap`](unsync::LazyMap)
//! (or [`LazyLockFn`](sync::LazyLockFn) to implement [`Sync`](std::marker::Sync)):
//! ```
//! # use std::cell::Cell;
//! # use rand::{Rng, thread_rng};
//! # use safe_once_map::cell::LazyCellMap;
//! let memoized = LazyCellMap::new(|x:u8| thread_rng().gen::<u32>());
//! assert_eq!(memoized(1), memoized(1));
//! assert_eq!(memoized(2), memoized(2));
//! assert_ne!(memoized(1), memoized(2));
//! ```
//!
//! ## Method
//! A method can be memoized with [`OnceCellMap`](unsync::OnceCellMap)
//! (or [`OnceLockMap`](sync::OnceLockMap) to implement [`Sync`](std::marker::Sync)):
//! ```
//! # use std::cell::Cell;
//! # use rand::{Rng, thread_rng};
//! # use safe_once::cell::OnceCell;
//! # use safe_once_map::cell::OnceCellMap;
//! struct Fibonacci { memo: OnceCellMap<usize,usize> };
//! impl Fibonacci {
//!     fn fib(&self, x: usize) -> usize{
//!         *self.memo[&x].get_or_init(||{
//!             if x == 0 || x == 1 {
//!                 1
//!             }else{
//!                 self.fib(x - 1) + self.fib(x - 2)
//!             }
//!         })
//!     }
//! }
//! let fib = Fibonacci { memo: OnceCellMap::new() };
//! assert_eq!(fib.fib(0), 1);
//! assert_eq!(fib.fib(1), 1);
//! assert_eq!(fib.fib(2), 2);
//! ```
//!
//! ## Top-level `fn`
//! A top-level `fn` can be memoized with [`OnceLockMap`](sync::OnceLockMap):
//! ```
//! # use rand::{Rng, thread_rng};
//! # use safe_once::sync::{LazyLock, OnceLock};
//! # use safe_once_map::sync::OnceLockMap;
//! fn memoized(x: u8) -> u32{
//!     static MAP : LazyLock<OnceLockMap<u8, u32>> = LazyLock::new(Default::default);
//!     *MAP[&x].get_or_init(|| {
//!         thread_rng().gen()
//!     })
//! }
//! assert_eq!(memoized(0), memoized(0));
//! assert_eq!(memoized(1), memoized(1));
//! assert_ne!(memoized(0), memoized(1));
//! ```

pub mod cell;

pub mod sync;

// pub mod once_map;
pub mod util;
pub mod api;
