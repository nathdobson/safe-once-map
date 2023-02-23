use std::borrow::{Borrow, Cow};
use std::cell::{Ref, RefCell};
use std::collections::hash_map::{RandomState, RawEntryMut, RawVacantEntryMut};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem;
use std::ops::Index;
use colosseum::unsync::Arena;
use safe_once::unsync::OnceCell;
use crate::raw_mutex_cell::RawMutexCell;
use crate::stable_map::{StableMap, UnboundedRef};
use crate::raw_map::RawMap;

pub type OnceCellMap<K, V, RM = RawMutexCell, M = HashMap<K, UnboundedRef<OnceCell<V>>>> = StableMap<K, OnceCell<V>, RM, M>;
