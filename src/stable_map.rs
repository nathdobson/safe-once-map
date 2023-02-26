use std::borrow::{Borrow, Cow};
use std::hash::Hash;

pub trait StableMap {
    type Key: Hash + Eq;
    type Value;
    fn get_or_insert<Q>(&self, key: Cow<Q>) -> &Self::Value where Q: ?Sized + Hash + Eq + ToOwned<Owned=Self::Key>, Self::Key: Borrow<Q>;
}
