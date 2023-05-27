use std::borrow::{Borrow, Cow};
use std::hash::Hash;
use std::ops::Index;

pub trait StableMapImpl {
    type Key: Hash + Eq;
    type Value;
    fn get_or_insert<Q>(&self, key: Cow<Q>) -> &Self::Value where Q: ?Sized + Hash + Eq + ToOwned<Owned=Self::Key>, Self::Key: Borrow<Q>;
}

#[derive(Default)]
pub struct StableMap<M>(pub M);

impl<'q, M, Q> Index<&'q Q> for StableMap<M>
    where Q: ?Sized + Hash + Eq + ToOwned<Owned=M::Key>,
          M::Key: Borrow<Q>,
          M: StableMapImpl {
    type Output = M::Value;
    fn index(&self, index: &'q Q) -> &Self::Output {
        self.0.get_or_insert(Cow::Borrowed(index))
    }
}